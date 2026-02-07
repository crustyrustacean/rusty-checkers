// backend/src/routes/health_check.rs

// dependencies
use crate::response::ApiResponse;

// health_check endpoint which returns a 200 OK response and empty body
pub async fn health_check() -> ApiResponse<()> {
    ApiResponse::success(())
}