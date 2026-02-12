// backend/src/routes/web_socket.rs

// dependencies
use crate::game_server::{
    GameId, GameServer, GameSession, PlayerSender, TournamentCode, TournamentConnection,
};
use crate::tournament::manager::MatchStartAction;
use crate::tournament::manager::TournamentManager;
use checkers_common::{
    AiDifficulty, AiPlayer, ClientMessage, Game, MinimaxAi, MoveResult, Player, RandomAi,
    ServerMessage,
};
use rama::http::ws::Message;
use rama::http::ws::handshake::server::ServerWebSocket;
use rama::telemetry::tracing;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Per-connection state for tracking tournament membership.
struct ConnectionState {
    game_id: Option<GameId>,
    player: Option<Player>,
    tournament_player_id: Option<Uuid>,
    tournament_code: Option<TournamentCode>,
}

pub async fn game_handler(
    mut ws: ServerWebSocket,
    game_server: Arc<GameServer>,
) -> Result<(), std::convert::Infallible> {
    tracing::info!("Client connected");

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let mut conn = ConnectionState {
        game_id: None,
        player: None,
        tournament_player_id: None,
        tournament_code: None,
    };

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if ws.send_message(Message::text(msg)).await.is_err() {
                    break;
                }
            }
            result = ws.recv_message() => {
                match result {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::JoinGame) => {
                                if let Some((game_id, player)) = handle_join_game(tx.clone(), &game_server).await {
                                    conn.game_id = Some(game_id);
                                    conn.player = Some(player);
                                }
                            }
                            Ok(ClientMessage::PlayVsAI { difficulty }) => {
                                if let Some((game_id, player)) = handle_play_vs_ai(tx.clone(), difficulty, &game_server).await {
                                    conn.game_id = Some(game_id);
                                    conn.player = Some(player);
                                }
                            }
                            Ok(ClientMessage::MakeMove { start, end }) => {
                                tracing::info!("MakeMove: {:?} -> {:?}", start, end);
                                if let (Some(gid), Some(player)) = (&conn.game_id, &conn.player) {
                                    handle_make_move(gid, player, start, end, &tx, &game_server).await;
                                } else {
                                    send_error(&tx, "Not in a game");
                                }
                            }
                            Ok(ClientMessage::PlayAgain) => {
                                if let Some(gid) = &conn.game_id {
                                    handle_play_again(gid, &game_server).await;
                                }
                            }
                            Ok(ClientMessage::CreateTournament { host_name }) => {
                                if let Some((code, player_id)) = handle_create_tournament(
                                    tx.clone(), host_name, &game_server,
                                ).await {
                                    conn.tournament_code = Some(code);
                                    conn.tournament_player_id = Some(player_id);
                                }
                            }
                            Ok(ClientMessage::JoinTournament { code, name }) => {
                                if let Some((code, player_id)) = handle_join_tournament(
                                    tx.clone(), code, name, &game_server,
                                ).await {
                                    conn.tournament_code = Some(code);
                                    conn.tournament_player_id = Some(player_id);
                                }
                            }
                            Ok(ClientMessage::StartTournament) => {
                                if let Some(code) = &conn.tournament_code {
                                    if let Some(player_id) = conn.tournament_player_id {
                                        handle_start_tournament(code, player_id, &game_server).await;
                                    }
                                } else {
                                    send_error(&tx, "Not in a tournament");
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Invalid message: {}", e);
                                send_error(&tx, &format!("Invalid message: {}", e));
                            }
                        }
                    }
                    Ok(_) => {} // Ignore binary, ping, pong
                    Err(_) => break,
                }
            }
        }
    }

    tracing::info!("Client disconnected");
    handle_disconnect(&conn, &game_server).await;
    Ok(())
}

// --- Helper ---

fn send_error(tx: &PlayerSender, msg: &str) {
    let _ = tx.send(serde_json::to_string(&ServerMessage::Error(msg.into())).unwrap());
}

fn send_msg(tx: &PlayerSender, msg: &ServerMessage) {
    let _ = tx.send(serde_json::to_string(msg).unwrap());
}

// --- Standard Game Handlers (unchanged logic) ---

async fn handle_join_game(
    tx: PlayerSender,
    game_server: &Arc<GameServer>,
) -> Option<(GameId, Player)> {
    let mut waiting = game_server.waiting_player.write().await;

    if let Some(game_id) = waiting.take() {
        tracing::info!("Player joining existing game: {}", game_id);

        let mut games = game_server.games.write().await;
        if let Some(session) = games.get_mut(&game_id) {
            session.light_player = Some(tx.clone());

            let _ =
                tx.send(serde_json::to_string(&ServerMessage::GameStarted(Player::Light)).unwrap());

            if let Some(dark_tx) = &session.dark_player {
                let _ = dark_tx.send(
                    serde_json::to_string(&ServerMessage::GameStarted(Player::Dark)).unwrap(),
                );
            }

            let game_state =
                serde_json::to_string(&ServerMessage::GameState(session.game.to_json())).unwrap();
            if let Some(dark_tx) = &session.dark_player {
                let _ = dark_tx.send(game_state.clone());
            }
            let _ = tx.send(game_state);

            return Some((game_id, Player::Light));
        }
        None
    } else {
        let game_id: GameId = Uuid::new_v4().to_string();
        tracing::info!("Player creating new game: {}", game_id);

        let session = GameSession {
            game: Box::new(Game::new()),
            dark_player: Some(tx.clone()),
            light_player: None,
            ai_opponent: None,
            ai_color: None,
            tournament_code: None,
        };
        game_server
            .games
            .write()
            .await
            .insert(game_id.clone(), session);
        *waiting = Some(game_id.clone());

        let _ = tx.send(serde_json::to_string(&ServerMessage::GamePending).unwrap());
        Some((game_id, Player::Dark))
    }
}

async fn handle_play_vs_ai(
    tx: PlayerSender,
    difficulty: AiDifficulty,
    game_server: &Arc<GameServer>,
) -> Option<(GameId, Player)> {
    let game_id: GameId = Uuid::new_v4().to_string();
    tracing::info!(
        "Player starting AI game (difficulty: {:?}): {}",
        difficulty,
        game_id
    );

    let ai: Box<dyn AiPlayer + Send + Sync> = match difficulty {
        AiDifficulty::Easy => Box::new(RandomAi),
        AiDifficulty::Medium => Box::new(MinimaxAi::new(4)),
        AiDifficulty::Hard => Box::new(MinimaxAi::new(6)),
    };

    let session = GameSession {
        game: Box::new(Game::new()),
        dark_player: Some(tx.clone()),
        light_player: None,
        ai_opponent: Some(ai),
        ai_color: Some(Player::Light),
        tournament_code: None,
    };

    // Capture initial state before moving session into the map
    let initial_state = session.game.to_json();

    game_server
        .games
        .write()
        .await
        .insert(game_id.clone(), session);

    let _ = tx.send(serde_json::to_string(&ServerMessage::GameStarted(Player::Dark)).unwrap());
    let _ = tx.send(serde_json::to_string(&ServerMessage::GameState(initial_state)).unwrap());

    Some((game_id, Player::Dark))
}

async fn handle_make_move(
    game_id: &GameId,
    player: &Player,
    start: (usize, usize),
    end: (usize, usize),
    tx: &PlayerSender,
    game_server: &Arc<GameServer>,
) {
    let (tournament_code, winner) = {
        let mut games = game_server.games.write().await;
        let Some(session) = games.get_mut(game_id) else {
            send_error(tx, "Game not found");
            return;
        };

        if session.game.current_player() != *player {
            send_error(tx, "Not your turn");
            return;
        }

        let result = session.game.apply_move(start, end);

        match result {
            MoveResult::InvalidMove(reason) => {
                send_error(tx, &reason);
                return;
            }
            _ => {
                broadcast_game_state(session);

                // AI turn handling
                if result == MoveResult::TurnComplete
                    && session.game.winner().is_none()
                    && session.ai_opponent.is_some()
                    && let (Some(ai), Some(ai_color)) = (&session.ai_opponent, &session.ai_color)
                {
                    if session.game.current_player() == *ai_color {
                        let ai_color_val = ai_color.clone();

                        loop {
                            let Some((ai_start, ai_end)) =
                                ai.select_move(&*session.game, &ai_color_val)
                            else {
                                break;
                            };

                            tracing::info!("AI move: {:?} -> {:?}", ai_start, ai_end);
                            let ai_result = session.game.apply_move(ai_start, ai_end);
                            broadcast_game_state(session);

                            match ai_result {
                                MoveResult::TurnComplete => break,
                                MoveResult::ContinueJump(_, _) => continue,
                                MoveResult::GameWon(_) => break,
                                MoveResult::InvalidMove(e) => {
                                    tracing::error!("AI attempted invalid move: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }

                // Check if this game has a winner and belongs to a tournament
                let winner = session.game.winner();
                let tc = session.tournament_code.clone();
                (tc, winner)
            }
        }
    };

    // If a tournament game just ended, advance the bracket
    if let (Some(code), Some(winner_color)) = (tournament_code, winner) {
        handle_tournament_game_end(game_id, &winner_color, &code, game_server).await;
    }
}

/// Send the current game state to all connected players in the session.
fn broadcast_game_state(session: &GameSession) {
    let game_state =
        serde_json::to_string(&ServerMessage::GameState(session.game.to_json())).unwrap();

    tracing::info!("Broadcasting game state to players");

    if let Some(dark_tx) = &session.dark_player {
        let _ = dark_tx.send(game_state.clone());
    }
    if let Some(light_tx) = &session.light_player {
        let _ = light_tx.send(game_state);
    }
}

async fn handle_play_again(game_id: &GameId, game_server: &Arc<GameServer>) {
    let mut games = game_server.games.write().await;
    let Some(session) = games.get_mut(game_id) else {
        return;
    };

    session.game = Box::new(Game::new());
    tracing::info!("Game {} reset for play again", game_id);
    broadcast_game_state(session);
}

// --- Tournament Handlers ---

/// Generate a short, human-readable room code.
fn generate_room_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..4).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

async fn handle_create_tournament(
    tx: PlayerSender,
    host_name: String,
    game_server: &Arc<GameServer>,
) -> Option<(TournamentCode, Uuid)> {
    let tm = TournamentManager::new(String::new(), host_name);
    let host_id = tm.host_id();

    // Insert with a unique room code (retry on collision)
    let (code, view) = {
        let mut tournaments = game_server.tournaments.write().await;
        let code = loop {
            let candidate = generate_room_code();
            if !tournaments.contains_key(&candidate) {
                break candidate;
            }
        };
        let mut tm = tm;
        // Set the code on the manager after generating it
        tm.set_code(code.clone());
        let view = tm.view();
        tournaments.insert(code.clone(), tm);
        (code, view)
    };

    tracing::info!("Creating tournament with code: {}", code);

    game_server.tournament_connections.write().await.insert(
        host_id,
        TournamentConnection {
            tournament_code: code.clone(),
            player_id: host_id,
            tx: tx.clone(),
        },
    );

    send_msg(&tx, &ServerMessage::TournamentCreated { code: code.clone() });
    send_msg(&tx, &ServerMessage::TournamentUpdate(view));

    Some((code, host_id))
}

async fn handle_join_tournament(
    tx: PlayerSender,
    code: String,
    name: String,
    game_server: &Arc<GameServer>,
) -> Option<(TournamentCode, Uuid)> {
    let code = code.to_uppercase();

    // Scope the tournaments lock so it's released before acquiring tournament_connections
    let (player_id, view) = {
        let mut tournaments = game_server.tournaments.write().await;
        let Some(tm) = tournaments.get_mut(&code) else {
            send_error(&tx, "Tournament not found");
            return None;
        };

        let player_id = match tm.add_player(name) {
            Ok(id) => id,
            Err(e) => {
                send_error(&tx, &e);
                return None;
            }
        };

        tracing::info!("Player {} joined tournament {}", player_id, code);
        (player_id, tm.view())
    }; // tournaments lock dropped here

    game_server.tournament_connections.write().await.insert(
        player_id,
        TournamentConnection {
            tournament_code: code.clone(),
            player_id,
            tx: tx.clone(),
        },
    );

    broadcast_tournament_view(&view, game_server).await;

    Some((code, player_id))
}

async fn handle_start_tournament(
    code: &str,
    requester_id: Uuid,
    game_server: &Arc<GameServer>,
) {
    let actions = {
        let mut tournaments = game_server.tournaments.write().await;
        let Some(tm) = tournaments.get_mut(code) else {
            return;
        };

        // Only the host can start
        if tm.host_id() != requester_id {
            tracing::warn!("Non-host tried to start tournament {}", code);
            return;
        }

        match tm.start() {
            Ok(actions) => actions,
            Err(e) => {
                tracing::warn!("Failed to start tournament {}: {}", code, e);
                // Drop tournaments lock before acquiring tournament_connections
                drop(tournaments);
                let conns = game_server.tournament_connections.read().await;
                if let Some(conn) = conns.get(&requester_id) {
                    send_error(&conn.tx, &e);
                }
                return;
            }
        }
    };

    // Create game sessions for each ready match
    for action in &actions {
        create_tournament_game_session(code, action, game_server).await;
    }

    // Broadcast updated view
    let tournaments = game_server.tournaments.read().await;
    if let Some(tm) = tournaments.get(code) {
        let view = tm.view();
        drop(tournaments);
        broadcast_tournament_view(&view, game_server).await;
    }
}

/// Create a GameSession for a tournament match and notify both players.
async fn create_tournament_game_session(
    tournament_code: &str,
    action: &MatchStartAction,
    game_server: &Arc<GameServer>,
) {
    let conns = game_server.tournament_connections.read().await;

    let dark_tx = conns.get(&action.player1_id).map(|c| c.tx.clone());
    let light_tx = conns.get(&action.player2_id).map(|c| c.tx.clone());
    drop(conns);

    let session = GameSession {
        game: Box::new(Game::new()),
        dark_player: dark_tx.clone(),
        light_player: light_tx.clone(),
        ai_opponent: None,
        ai_color: None,
        tournament_code: Some(tournament_code.to_string()),
    };

    // Capture initial state before moving session into the map
    let initial_state = session.game.to_json();

    game_server
        .games
        .write()
        .await
        .insert(action.game_id.clone(), session);

    // Notify player1 (Dark)
    if let Some(tx) = &dark_tx {
        send_msg(tx, &ServerMessage::MatchStart {
            game_id: action.game_id.clone(),
            opponent_name: action.player2_name.clone(),
        });
        send_msg(tx, &ServerMessage::GameStarted(Player::Dark));
        send_msg(tx, &ServerMessage::GameState(initial_state.clone()));
    }

    // Notify player2 (Light)
    if let Some(tx) = &light_tx {
        send_msg(tx, &ServerMessage::MatchStart {
            game_id: action.game_id.clone(),
            opponent_name: action.player1_name.clone(),
        });
        send_msg(tx, &ServerMessage::GameStarted(Player::Light));
        send_msg(tx, &ServerMessage::GameState(initial_state));
    }

    tracing::info!(
        "Tournament match started: {} vs {} (game_id: {})",
        action.player1_name,
        action.player2_name,
        action.game_id
    );
}

/// Called when a tournament game ends. Advances the bracket and potentially starts next match.
async fn handle_tournament_game_end(
    game_id: &str,
    winner_color: &Player,
    tournament_code: &str,
    game_server: &Arc<GameServer>,
) {
    // Determine the winner's player ID and advance bracket in a single write lock
    let (next_action, view) = {
        let mut tournaments = game_server.tournaments.write().await;
        let Some(tm) = tournaments.get_mut(tournament_code) else {
            return;
        };

        let Some(match_id) = tm.find_match_by_game_id(game_id) else {
            return;
        };

        // Use find_match to look up player IDs directly (avoids constructing full view)
        let winner_id = {
            let Some(bracket_match) = tm.find_match(match_id) else {
                return;
            };
            // In tournament matches, Dark = player1, Light = player2
            let id = match winner_color {
                Player::Dark => bracket_match.player1_id,
                Player::Light => bracket_match.player2_id,
            };
            let Some(id) = id else { return };
            id
        };

        let next_action = match tm.advance(match_id, winner_id) {
            Ok(action) => action,
            Err(e) => {
                tracing::error!("Failed to advance tournament: {}", e);
                return;
            }
        };

        let view = tm.view();
        (next_action, view)
    }; // tournaments lock dropped

    // If a new match is ready, create its game session
    if let Some(action) = &next_action {
        create_tournament_game_session(tournament_code, action, game_server).await;
    }

    // Broadcast the updated bracket
    broadcast_tournament_view(&view, game_server).await;
}

/// Send tournament view to all connected tournament participants.
async fn broadcast_tournament_view(
    view: &checkers_common::tournament::TournamentView,
    game_server: &Arc<GameServer>,
) {
    let msg = ServerMessage::TournamentUpdate(view.clone());
    let conns = game_server.tournament_connections.read().await;

    for player in &view.players {
        if let Some(conn) = conns.get(&player.id) {
            send_msg(&conn.tx, &msg);
        }
    }
}

// --- Disconnect Handling ---

async fn handle_disconnect(conn: &ConnectionState, game_server: &Arc<GameServer>) {
    // Handle tournament disconnect + forfeit
    if let (Some(code), Some(player_id)) = (&conn.tournament_code, conn.tournament_player_id) {
        let forfeit_action = {
            let mut tournaments = game_server.tournaments.write().await;
            if let Some(tm) = tournaments.get_mut(code) {
                tm.disconnect_player(player_id);

                // Check if player is in an active match
                if let Some(active_match) = tm.find_active_match_for_player(player_id) {
                    let match_id = active_match.id;
                    match tm.forfeit(match_id, player_id) {
                        Ok(action) => action,
                        Err(e) => {
                            tracing::error!("Forfeit failed: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(action) = &forfeit_action {
            create_tournament_game_session(code, action, game_server).await;
        }

        // Broadcast updated bracket
        let tournaments = game_server.tournaments.read().await;
        if let Some(tm) = tournaments.get(code.as_str()) {
            let view = tm.view();
            drop(tournaments);
            broadcast_tournament_view(&view, game_server).await;
        }

        // Remove this player's connection
        game_server
            .tournament_connections
            .write()
            .await
            .remove(&player_id);
    }

    // Notify opponent of disconnect in regular games
    if let Some(game_id) = &conn.game_id {
        let games = game_server.games.read().await;
        if let Some(session) = games.get(game_id) {
            let disconnect_msg =
                serde_json::to_string(&ServerMessage::OpponentDisconnected).unwrap();
            if let Some(dark_tx) = &session.dark_player {
                let _ = dark_tx.send(disconnect_msg.clone());
            }
            if let Some(light_tx) = &session.light_player {
                let _ = light_tx.send(disconnect_msg);
            }
        }
    }
}
