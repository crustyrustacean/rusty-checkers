// backend/src/routes/web_socket.rs

// dependencies
use crate::game_server::{GameId, GameServer, GameSession, PlayerSender};
use checkers_common::{
    AiDifficulty, AiPlayer, ClientMessage, Game, MinimaxAi, Player, RandomAi, ServerMessage,
};
use rama::http::ws::Message;
use rama::http::ws::handshake::server::ServerWebSocket;
use rama::telemetry::tracing;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn game_handler(
    mut ws: ServerWebSocket,
    game_server: Arc<GameServer>,
) -> Result<(), std::convert::Infallible> {
    tracing::info!("Client connected");

    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Track which game this player is in
    let mut my_game_id: Option<GameId> = None;
    let mut my_player: Option<Player> = None;

    loop {
        tokio::select! {
            // Messages from the channel (from other players or server)
            Some(msg) = rx.recv() => {
                if ws.send_message(Message::text(msg)).await.is_err() {
                    break;
                }
            }
            // Messages from this client's WebSocket
            result = ws.recv_message() => {
                match result {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::JoinGame) => {
                                if let Some((game_id, player)) = handle_join_game(tx.clone(), &game_server).await {
                                    my_game_id = Some(game_id);
                                    my_player = Some(player);
                                }
                            }
                            Ok(ClientMessage::PlayVsAI { difficulty }) => {
                                if let Some((game_id, player)) = handle_play_vs_ai(tx.clone(), difficulty, &game_server).await {
                                    my_game_id = Some(game_id);
                                    my_player = Some(player);
                                }
                            }
                            Ok(ClientMessage::MakeMove { start, end }) => {
                                tracing::info!("MakeMove: {:?} -> {:?}", start, end);
                                if let (Some(gid), Some(player)) = (&my_game_id, &my_player) {
                                    handle_make_move(gid, player, start, end, &tx, &game_server).await;
                                } else {
                                    let _ = tx.send(serde_json::to_string(&ServerMessage::Error("Not in a game".into())).unwrap());
                                }
                            }
                            Ok(ClientMessage::PlayAgain) => {
                                if let Some(gid) = &my_game_id {
                                    handle_play_again(gid, &game_server).await;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Invalid message: {}", e);
                                let error = ServerMessage::Error(format!("Invalid message: {}", e));
                                let _ = tx.send(serde_json::to_string(&error).unwrap());
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
    // TODO: cleanup game state on disconnect
    Ok(())
}

async fn handle_join_game(
    tx: PlayerSender,
    game_server: &Arc<GameServer>,
) -> Option<(GameId, Player)> {
    let mut waiting = game_server.waiting_player.write().await;

    if let Some(game_id) = waiting.take() {
        // Join existing game as Light
        tracing::info!("Player joining existing game: {}", game_id);

        let mut games = game_server.games.write().await;
        if let Some(session) = games.get_mut(&game_id) {
            session.light_player = Some(tx.clone());

            // Notify this player they're Light
            let _ =
                tx.send(serde_json::to_string(&ServerMessage::GameStarted(Player::Light)).unwrap());

            // Notify Dark player that game started
            if let Some(dark_tx) = &session.dark_player {
                let _ = dark_tx.send(
                    serde_json::to_string(&ServerMessage::GameStarted(Player::Dark)).unwrap(),
                );
            }

            // Send initial game state to both
            let game_state =
                serde_json::to_string(&ServerMessage::GameState(session.game.clone())).unwrap();
            if let Some(dark_tx) = &session.dark_player {
                let _ = dark_tx.send(game_state.clone());
            }
            let _ = tx.send(game_state);

            return Some((game_id, Player::Light));
        }
        None
    } else {
        // Create new game and wait
        let game_id: GameId = Uuid::new_v4().to_string();
        tracing::info!("Player creating new game: {}", game_id);

        let session = GameSession {
            game: Game::new(),
            dark_player: Some(tx.clone()),
            light_player: None,
            ai_opponent: None,
            ai_color: None,
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

/// Handle a request to play against the AI.
///
/// Creates a new game session with the human as Dark and the AI as Light.
/// The game starts immediately without waiting for another player.
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
        game: Game::new(),
        dark_player: Some(tx.clone()),
        light_player: None,
        ai_opponent: Some(ai),
        ai_color: Some(Player::Light),
    };

    game_server
        .games
        .write()
        .await
        .insert(game_id.clone(), session);

    // Notify the human they are Dark
    let _ = tx.send(serde_json::to_string(&ServerMessage::GameStarted(Player::Dark)).unwrap());

    // Send initial game state
    let games = game_server.games.read().await;
    if let Some(session) = games.get(&game_id) {
        let game_state =
            serde_json::to_string(&ServerMessage::GameState(session.game.clone())).unwrap();
        let _ = tx.send(game_state);
    }

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
    let mut games = game_server.games.write().await;
    let Some(session) = games.get_mut(game_id) else {
        let _ =
            tx.send(serde_json::to_string(&ServerMessage::Error("Game not found".into())).unwrap());
        return;
    };

    // Check it's this player's turn
    if session.game.current_player != *player {
        let _ =
            tx.send(serde_json::to_string(&ServerMessage::Error("Not your turn".into())).unwrap());
        return;
    }

    // Find the piece at start position
    let Some(piece) = session
        .game
        .pieces
        .iter()
        .find(|p| p.row == start.0 && p.col == start.1)
        .cloned()
    else {
        let _ = tx.send(
            serde_json::to_string(&ServerMessage::Error("No piece at start position".into()))
                .unwrap(),
        );
        return;
    };

    // Verify piece belongs to current player
    if piece.owner != *player {
        let _ =
            tx.send(serde_json::to_string(&ServerMessage::Error("Not your piece".into())).unwrap());
        return;
    }

    // Check if move is valid
    let valid_moves = session.game.valid_moves(&piece);
    if !valid_moves.contains(&end) {
        let _ =
            tx.send(serde_json::to_string(&ServerMessage::Error("Invalid move".into())).unwrap());
        return;
    }

    // Execute the move
    execute_move(&mut session.game, start, end);

    // Broadcast state after human move
    broadcast_game_state(session);

    // If AI opponent exists and it's now the AI's turn, make the AI move
    if let (Some(ai), Some(ai_color)) = (&session.ai_opponent, &session.ai_color)
        && session.game.current_player == *ai_color
        && session.game.winner.is_none()
    {
        let ai_color = ai_color.clone();

        // AI move loop: handle multi-jumps
        loop {
            let Some((ai_start, ai_end)) = ai.select_move(&session.game, &ai_color) else {
                break;
            };

            tracing::info!("AI move: {:?} -> {:?}", ai_start, ai_end);
            execute_move(&mut session.game, ai_start, ai_end);

            // If the turn hasn't switched (multi-jump in progress), continue
            if session.game.current_player == ai_color && session.game.winner.is_none() {
                // Broadcast intermediate state so the human can see the multi-jump
                broadcast_game_state(session);
                continue;
            }
            break;
        }

        // Broadcast final state after AI completes its turn
        broadcast_game_state(session);
    }
}

/// Execute a single move on the game, handling captures, kinging, multi-jump
/// detection, turn switching, and winner detection.
fn execute_move(game: &mut Game, start: (usize, usize), end: (usize, usize)) {
    let piece = game
        .pieces
        .iter()
        .find(|p| p.row == start.0 && p.col == start.1)
        .cloned();

    let Some(piece) = piece else {
        return;
    };

    let was_jump = (end.0 as i32 - start.0 as i32).abs() == 2;
    game.advance(&piece, end.0, end.1);

    if was_jump {
        let captured_row = (start.0 + end.0) / 2;
        let captured_col = (start.1 + end.1) / 2;
        game.capture(captured_row, captured_col);
    }

    // Handle kinging
    let mut just_kinged = false;
    if let Some(p) = game
        .pieces
        .iter_mut()
        .find(|p| p.row == end.0 && p.col == end.1)
        && ((p.row == 0 && p.owner == Player::Light) || (p.row == 7 && p.owner == Player::Dark))
        && !p.is_kinged
    {
        p.is_kinged = true;
        just_kinged = true;
    }

    // Check for multi-jump
    let can_jump_again = if was_jump && !just_kinged {
        game.pieces
            .iter()
            .find(|p| p.row == end.0 && p.col == end.1)
            .map(|p| game.has_available_jumps(p))
            .unwrap_or(false)
    } else {
        false
    };

    if !can_jump_again {
        game.switch_turn();

        // Check for winner
        let current = game.current_player.clone();
        let has_moves = game
            .pieces
            .iter()
            .filter(|p| p.owner == current)
            .any(|p| !game.valid_moves(p).is_empty());

        if !has_moves {
            game.winner = Some(match current {
                Player::Dark => Player::Light,
                Player::Light => Player::Dark,
            });
        }
    }
}

/// Send the current game state to all connected players in the session.
fn broadcast_game_state(session: &GameSession) {
    let game_state =
        serde_json::to_string(&ServerMessage::GameState(session.game.clone())).unwrap();

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

    // Reset the game
    session.game = Game::new();

    tracing::info!("Game {} reset for play again", game_id);

    // Broadcast new state to both players
    broadcast_game_state(session);
}
