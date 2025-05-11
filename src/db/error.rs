use sqlx::error::{BoxDynError, Error as SqlxError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] SqlxError),

    #[error("Record not found")]
    NotFound,

    #[error("Duplicate record: {0}")]
    Duplicate(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Database error: {0}")]
    Other(BoxDynError),
}

impl From<BoxDynError> for DatabaseError {
    fn from(err: BoxDynError) -> Self {
        DatabaseError::Other(err)
    }
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
