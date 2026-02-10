// backend/src/routes/web_socket.rs

// dependencies
use crate::game_server::GameServer;
use rama::http::ws::handshake::server::ServerWebSocket;
use std::sync::Arc;
use rama::telemetry::tracing;

pub async fn echo_handler(mut ws: ServerWebSocket, game_server: Arc<GameServer>) -> Result<(), std::convert::Infallible> {
    tracing::info!("Connected! Active games: {}", game_server.games.read().await.len());

    while let Ok(msg) = ws.recv_message().await {
        if ws.send_message(msg).await.is_err() {
            break;
        }
    }
    Ok(())
}
