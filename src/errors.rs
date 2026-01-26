use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

/// Application-wide error type.
#[derive(Debug)]
pub enum AppError {
    Database(surrealdb::Error),
    ValueError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Database(err) => {
                // log the real error for seeing in the terminal/logs.
                error!("Database error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            AppError::ValueError(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}
