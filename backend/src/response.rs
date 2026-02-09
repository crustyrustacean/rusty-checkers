// backend/src/response.rs

// common API response type

// dependencies
use crate::errors::ApiError;
use chrono::{DateTime, Utc};
use rama::http::{
    StatusCode,
    response::Response,
    service::web::response::{IntoResponse, Json},
};
use serde::Serialize;

// convenience alias for results returned by handlers
pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;

// struct type to represent an API response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub status: u16,
    pub time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

// methods for the ApiResponse type
impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::success_with_status(StatusCode::OK, data)
    }

    pub fn success_with_status(status: StatusCode, data: T) -> Self {
        Self {
            success: true,
            message: Some("ok".into()),
            status: status.as_u16(),
            time: Utc::now(),
            data: Some(data),
        }
    }

    pub fn error(message: &str, status: StatusCode) -> Self {
        Self {
            success: false,
            message: Some(message.to_string()),
            status: status.as_u16(),
            time: Utc::now(),
            data: None,
        }
    }
}

// implement the IntoResponse trait for the ApiResponse type
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}
