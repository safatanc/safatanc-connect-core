use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: NaiveDateTime,
    pub refresh_token_expires_at: Option<NaiveDateTime>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: Option<serde_json::Value>,
    pub is_active: bool,
    pub last_activity_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: NaiveDateTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_info: Option<serde_json::Value>,
    pub is_active: bool,
    pub last_activity_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

// Implementation of From trait for converting from Session to SessionResponse
impl From<Session> for SessionResponse {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            user_id: session.user_id,
            expires_at: session.expires_at,
            ip_address: session.ip_address,
            user_agent: session.user_agent,
            device_info: session.device_info,
            is_active: session.is_active,
            last_activity_at: session.last_activity_at,
            created_at: session.created_at,
        }
    }
}
