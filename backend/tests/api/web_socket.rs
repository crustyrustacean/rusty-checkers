// tests/api/web_socket.rs

// dependencies
use crate::helpers::spawn_app;
use rama::http::StatusCode;

#[tokio::test]
async fn web_socket_endpoint_upgrades() {
    // Arrange
    let app = spawn_app().await;

    // Build a WebSocket upgrade request with required headers
    let request = rama::http::Request::builder()
        .method("GET")
        .uri("/api/v1/ws")
        .header("Host", "localhost")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(rama::http::Body::empty())
        .expect("Failed to build WebSocket upgrade request");

    // Act
    let response = app.serve(request).await;

    // Assert
    assert_eq!(StatusCode::SWITCHING_PROTOCOLS, response.status());
}
