// tests/api/health_check.rs

// dependencies
use crate::helpers::spawn_app;
use rama::http::StatusCode;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;

    // make a request to the /api/v1/health_check endpoint, with an empty request body
    let request = app.build_request("/api/v1/health_check", None);

    // Act
    let response = app.serve(request).await;

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}