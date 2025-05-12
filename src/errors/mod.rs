use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::db::error::DatabaseError;
use crate::models::response::error_response;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Authorization(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Database(e) => match e {
                DatabaseError::NotFound => {
                    (StatusCode::NOT_FOUND, "Resource not found".to_string())
                }
                DatabaseError::Duplicate(msg) => (StatusCode::CONFLICT, msg),
                DatabaseError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal database error occurred".to_string(),
                ),
            },
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::InvalidToken(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Unexpected(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        error_response(status, message)
    }
}

pub type AppResult<T> = Result<T, AppError>;
