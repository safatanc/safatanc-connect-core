use chrono::{Duration, Utc};
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::auth::token::{CreateVerificationTokenDto, VerificationToken};

#[derive(Clone)]
pub struct TokenRepository {
    pool: PgPool,
}

impl TokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Create a verification token (for email verification, password reset, etc.)
    pub async fn create(
        &self,
        dto: &CreateVerificationTokenDto,
        token: &str,
    ) -> DatabaseResult<VerificationToken> {
        // Calculate expiration time
        let expires_at = Utc::now() + Duration::seconds(dto.expires_in);

        sqlx::query_as!(
            VerificationToken,
            r#"
            INSERT INTO verification_tokens (
                user_id, token, type, expires_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, user_id, token, type as "token_type", expires_at, used_at,
                created_at, updated_at
            "#,
            dto.user_id,
            token,
            dto.token_type,
            expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "verification_tokens_token_key" => {
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

    // Find a token by its value and type
    pub async fn find_by_token(
        &self,
        token: &str,
        token_type: &str,
    ) -> DatabaseResult<VerificationToken> {
        let token = sqlx::query_as!(
            VerificationToken,
            r#"
            SELECT 
                id, user_id, token, type as "token_type", expires_at, used_at,
                created_at, updated_at
            FROM verification_tokens
            WHERE token = $1 AND type = $2
            "#,
            token,
            token_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        token.ok_or(DatabaseError::NotFound)
    }

    // Find active tokens by user ID and type
    pub async fn find_active_by_user_and_type(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> DatabaseResult<Vec<VerificationToken>> {
        let tokens = sqlx::query_as!(
            VerificationToken,
            r#"
            SELECT 
                id, user_id, token, type as "token_type", expires_at, used_at,
                created_at, updated_at
            FROM verification_tokens
            WHERE user_id = $1 AND type = $2 
            AND used_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
            user_id,
            token_type
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(tokens)
    }

    // Mark a token as used
    pub async fn mark_as_used(&self, token_id: Uuid) -> DatabaseResult<VerificationToken> {
        let now = Utc::now();

        let token = sqlx::query_as!(
            VerificationToken,
            r#"
            UPDATE verification_tokens
            SET
                used_at = $1,
                updated_at = NOW()
            WHERE id = $2 AND used_at IS NULL
            RETURNING 
                id, user_id, token, type as "token_type", expires_at, used_at,
                created_at, updated_at
            "#,
            now,
            token_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        token.ok_or(DatabaseError::NotFound)
    }

    // Invalidate all tokens of a certain type for a user (e.g., invalidate all password reset tokens)
    pub async fn invalidate_by_user_and_type(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> DatabaseResult<PgQueryResult> {
        let now = Utc::now();

        sqlx::query!(
            r#"
            UPDATE verification_tokens
            SET
                used_at = $1,
                updated_at = NOW()
            WHERE user_id = $2 AND type = $3 AND used_at IS NULL
            "#,
            now,
            user_id,
            token_type
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    // Delete expired tokens
    pub async fn delete_expired(&self) -> DatabaseResult<PgQueryResult> {
        sqlx::query!(
            r#"
            DELETE FROM verification_tokens
            WHERE expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    // Verify if a token is valid
    pub async fn verify_token(
        &self,
        token: &str,
        token_type: &str,
    ) -> DatabaseResult<VerificationToken> {
        let token = sqlx::query_as!(
            VerificationToken,
            r#"
            SELECT 
                id, user_id, token, type as "token_type", expires_at, used_at,
                created_at, updated_at
            FROM verification_tokens
            WHERE token = $1 AND type = $2 
            AND used_at IS NULL AND expires_at > NOW()
            "#,
            token,
            token_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        token.ok_or(DatabaseError::NotFound)
    }
}
