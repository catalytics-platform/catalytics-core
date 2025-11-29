use crate::app_error::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Error: {}", self);

        match self {
            AppError::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
