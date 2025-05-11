use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub token: String,
    pub token_type: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct VerificationTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub token_type: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateVerificationTokenDto {
    pub user_id: Option<Uuid>,
    pub token_type: String,
    pub expires_in: i64, // seconds
}

#[derive(Debug, Deserialize)]
pub struct VerifyTokenDto {
    pub token: String,
    pub token_type: String,
}

// Token type constants
pub const TOKEN_TYPE_EMAIL_VERIFICATION: &str = "email_verification";
pub const TOKEN_TYPE_PASSWORD_RESET: &str = "password_reset";

// Implementation of From trait for converting from VerificationToken to VerificationTokenResponse
impl From<VerificationToken> for VerificationTokenResponse {
    fn from(token: VerificationToken) -> Self {
        Self {
            id: token.id,
            token: token.token,
            token_type: token.token_type,
            expires_at: token.expires_at,
        }
    }
}
