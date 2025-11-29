use crate::app_error::AppError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    status: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Error: {}", self);

        let (status, message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database exception".to_string(),
            ),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let error_response = ErrorResponse {
            status: status.as_u16(),
            message,
        };

        (status, Json(error_response)).into_response()
    }
}
