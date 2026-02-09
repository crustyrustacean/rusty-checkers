// backend/src/routes/web_socket.rs

// dependencies
use rama::http::ws::handshake::server::ServerWebSocket;

pub async fn echo_handler(mut ws: ServerWebSocket) -> Result<(), std::convert::Infallible> {
    while let Ok(msg) = ws.recv_message().await {
        if ws.send_message(msg).await.is_err() {
            break;
        }
    }
    Ok(())
}
