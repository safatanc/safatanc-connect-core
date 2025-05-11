use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::auth::session::Session;

#[derive(Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Create a new session
    pub async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        refresh_token: Option<&str>,
        expires_at: NaiveDateTime,
        refresh_token_expires_at: Option<NaiveDateTime>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_info: Option<serde_json::Value>,
    ) -> DatabaseResult<Session> {
        sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (
                user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            "#,
            user_id,
            token,
            refresh_token,
            expires_at,
            refresh_token_expires_at,
            ip_address,
            user_agent,
            device_info
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "sessions_token_key" => {
                            DatabaseError::Duplicate("Token already exists".to_string())
                        }
                        _ => DatabaseError::ConnectionError(e),
                    }
                } else {
                    DatabaseError::ConnectionError(e)
                }
            } else {
                DatabaseError::ConnectionError(e)
            }
        })
    }

    // Find session by ID
    pub async fn find_by_id(&self, id: Uuid) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            SELECT 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            FROM sessions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Find session by token
    pub async fn find_by_token(&self, token: &str) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            SELECT 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            FROM sessions
            WHERE token = $1 AND is_active = true
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Find session by refresh token
    pub async fn find_by_refresh_token(&self, refresh_token: &str) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            SELECT 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            FROM sessions
            WHERE refresh_token = $1 AND is_active = true
            "#,
            refresh_token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Get all active sessions for a user
    pub async fn find_by_user_id(&self, user_id: Uuid) -> DatabaseResult<Vec<Session>> {
        let sessions = sqlx::query_as!(
            Session,
            r#"
            SELECT 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            FROM sessions
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(sessions)
    }

    // Update session last activity
    pub async fn update_activity(&self, id: Uuid) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            UPDATE sessions
            SET
                last_activity_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Refresh session token
    pub async fn refresh(
        &self,
        id: Uuid,
        new_token: &str,
        new_refresh_token: Option<&str>,
        new_expires_at: NaiveDateTime,
        new_refresh_token_expires_at: Option<NaiveDateTime>,
    ) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            UPDATE sessions
            SET
                token = $1,
                refresh_token = $2,
                expires_at = $3,
                refresh_token_expires_at = $4,
                last_activity_at = NOW(),
                updated_at = NOW()
            WHERE id = $5 AND is_active = true
            RETURNING 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            "#,
            new_token,
            new_refresh_token,
            new_expires_at,
            new_refresh_token_expires_at,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Deactivate a session (logout)
    pub async fn deactivate(&self, id: Uuid) -> DatabaseResult<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"
            UPDATE sessions
            SET
                is_active = false,
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, user_id, token, refresh_token, expires_at, refresh_token_expires_at,
                ip_address, user_agent, device_info, is_active, last_activity_at,
                created_at, updated_at
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        session.ok_or(DatabaseError::NotFound)
    }

    // Deactivate all sessions for a user (logout from all devices)
    pub async fn deactivate_all_for_user(&self, user_id: Uuid) -> DatabaseResult<PgQueryResult> {
        sqlx::query!(
            r#"
            UPDATE sessions
            SET
                is_active = false,
                updated_at = NOW()
            WHERE user_id = $1 AND is_active = true
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    // Deactivate expired sessions
    pub async fn deactivate_expired(&self) -> DatabaseResult<PgQueryResult> {
        sqlx::query!(
            r#"
            UPDATE sessions
            SET
                is_active = false,
                updated_at = NOW()
            WHERE (expires_at < NOW() OR refresh_token_expires_at < NOW()) AND is_active = true
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    // Count active sessions for a user
    pub async fn count_active_for_user(&self, user_id: Uuid) -> DatabaseResult<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM sessions
            WHERE user_id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(count.count.unwrap_or(0))
    }
}
