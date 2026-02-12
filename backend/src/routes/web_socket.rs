// backend/src/routes/web_socket.rs

// dependencies
use crate::game_server::{GameId, GameServer, GameSession, PlayerSender};
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
                                    // play_move() validates piece ownership and move legality internally.
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
                serde_json::to_string(&ServerMessage::GameState(session.game.to_json())).unwrap();
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
            game: Box::new(Game::new()),
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
            serde_json::to_string(&ServerMessage::GameState(session.game.to_json())).unwrap();
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

    // 1. Verify Requestor
    // Check if the WebSocket user (player) matches the current turn of the game engine.
    // We use the trait method .current_player() here.
    if session.game.current_player() != *player {
        let _ =
            tx.send(serde_json::to_string(&ServerMessage::Error("Not your turn".into())).unwrap());
        return;
    }

    // 2. Apply Move via Trait
    // session.game is now Box<dyn BoardGame>, so we call the interface method.
    let result = session.game.apply_move(start, end);

    match result {
        MoveResult::InvalidMove(reason) => {
            // Send the specific validation error back to the client
            let _ = tx.send(serde_json::to_string(&ServerMessage::Error(reason)).unwrap());
        }
        _ => {
            // Move was successful (TurnComplete, ContinueJump, or GameWon)
            // Broadcast the new generic JSON state to all players
            broadcast_game_state(session);

            // 3. AI Turn Handling
            // If the human finished their turn, and the game isn't over, trigger AI.
            if result == MoveResult::TurnComplete
                && session.game.winner().is_none()
                && session.ai_opponent.is_some()
            {
                if let (Some(ai), Some(ai_color)) = (&session.ai_opponent, &session.ai_color) {
                    // Verify it is indeed the AI's turn according to the engine
                    if session.game.current_player() == *ai_color {
                        let ai_color_val = ai_color.clone();

                        // AI Loop: Keep playing as long as it's a multi-jump scenario
                        loop {
                            // Pass the trait object (dereferenced) to the AI
                            let Some((ai_start, ai_end)) =
                                ai.select_move(&*session.game, &ai_color_val)
                            else {
                                break;
                            };

                            tracing::info!("AI move: {:?} -> {:?}", ai_start, ai_end);

                            // Apply AI move via trait
                            let ai_result = session.game.apply_move(ai_start, ai_end);

                            // Broadcast every step so the client sees individual jumps
                            broadcast_game_state(session);

                            match ai_result {
                                MoveResult::TurnComplete => break,          // AI finished turn
                                MoveResult::ContinueJump(_, _) => continue, // AI must jump again
                                MoveResult::GameWon(_) => break,            // AI won
                                MoveResult::InvalidMove(e) => {
                                    tracing::error!("AI attempted invalid move: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
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

    // Reset the game
    session.game = Box::new(Game::new());

    tracing::info!("Game {} reset for play again", game_id);

    // Broadcast new state to both players
    broadcast_game_state(session);
}
