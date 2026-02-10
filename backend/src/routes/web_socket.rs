// backend/src/routes/web_socket.rs

// dependencies
use crate::game_server::{GameId, GameServer, GameSession, PlayerSender};
use checkers_common::{ClientMessage, Game, Player, ServerMessage};
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
                                Ok(ClientMessage::MakeMove { start, end }) => {
                                    tracing::info!("MakeMove: {:?} -> {:?}", start, end);
                                    if let (Some(gid), Some(player)) = (&my_game_id, &my_player) {
            handle_make_move(gid, player, start, end, &tx, &game_server).await;
        } else {
            let _ = tx.send(serde_json::to_string(&ServerMessage::Error("Not in a game".into())).unwrap());
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
    let was_jump = (end.0 as i32 - start.0 as i32).abs() == 2;
    session.game.advance(&piece, end.0, end.1);

    if was_jump {
        let captured_row = (start.0 + end.0) / 2;
        let captured_col = (start.1 + end.1) / 2;
        session.game.capture(captured_row, captured_col);
    }

    // Handle kinging
    let mut just_kinged = false;
    if let Some(p) = session
        .game
        .pieces
        .iter_mut()
        .find(|p| p.row == end.0 && p.col == end.1)
    {
        if (p.row == 0 && p.owner == Player::Light) || (p.row == 7 && p.owner == Player::Dark) {
            if !p.is_kinged {
                p.is_kinged = true;
                just_kinged = true;
            }
        }
    }

    // Check for multi-jump
    let can_jump_again = if was_jump && !just_kinged {
        session
            .game
            .pieces
            .iter()
            .find(|p| p.row == end.0 && p.col == end.1)
            .map(|p| session.game.has_available_jumps(p))
            .unwrap_or(false)
    } else {
        false
    };

    if !can_jump_again {
        session.game.switch_turn();

        // Check for winner
        let current = session.game.current_player.clone();
        let has_moves = session
            .game
            .pieces
            .iter()
            .filter(|p| p.owner == current)
            .any(|p| !session.game.valid_moves(p).is_empty());

        if !has_moves {
            session.game.winner = Some(match current {
                Player::Dark => Player::Light,
                Player::Light => Player::Dark,
            });
        }
    }

    // Broadcast new state to both players
    let game_state =
        serde_json::to_string(&ServerMessage::GameState(session.game.clone())).unwrap();
    if let Some(dark_tx) = &session.dark_player {
        let _ = dark_tx.send(game_state.clone());
    }
    if let Some(light_tx) = &session.light_player {
        let _ = light_tx.send(game_state);
    }
}
