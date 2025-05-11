use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::user::{CreateUserDto, UpdateUserDto, User};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Create a new user
    pub async fn create(&self, dto: &CreateUserDto, password_hash: String) -> DatabaseResult<User> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                email, username, password_hash, full_name, avatar_url, 
                global_role, is_email_verified, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            "#,
            dto.email,
            dto.username,
            password_hash,
            dto.full_name,
            dto.avatar_url,
            "user", // Default role
            false,  // Email not verified by default
            true,   // User active by default
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "users_email_key" => {
                            DatabaseError::Duplicate("Email already exists".to_string())
                        }
                        "users_username_key" => {
                            DatabaseError::Duplicate("Username already exists".to_string())
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

    // Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Find user by email
    pub async fn find_by_email(&self, email: &str) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Find user by username
    pub async fn find_by_username(&self, username: &str) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            FROM users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Get all users with pagination
    pub async fn find_all(&self, limit: i64, offset: i64) -> DatabaseResult<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            FROM users
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(users)
    }

    // Count all users
    pub async fn count(&self) -> DatabaseResult<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM users
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(count.count.unwrap_or(0))
    }

    // Update user
    pub async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> DatabaseResult<User> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                username = COALESCE($1, username),
                full_name = COALESCE($2, full_name),
                avatar_url = COALESCE($3, avatar_url),
                is_active = COALESCE($4, is_active),
                updated_at = now()
            WHERE id = $5 AND deleted_at IS NULL
            RETURNING 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            "#,
            dto.username,
            dto.full_name,
            dto.avatar_url,
            dto.is_active,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "users_username_key" => {
                            DatabaseError::Duplicate("Username already exists".to_string())
                        }
                        _ => DatabaseError::ConnectionError(e),
                    }
                } else {
                    DatabaseError::ConnectionError(e)
                }
            } else {
                DatabaseError::ConnectionError(e)
            }
        })?
        .ok_or(DatabaseError::NotFound)
    }

    // Update password
    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                password_hash = $1,
                updated_at = now()
            WHERE id = $2 AND deleted_at IS NULL
            RETURNING 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            "#,
            password_hash,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Update email verification status
    pub async fn update_email_verification(
        &self,
        id: Uuid,
        is_verified: bool,
    ) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                is_email_verified = $1,
                updated_at = now()
            WHERE id = $2 AND deleted_at IS NULL
            RETURNING 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            "#,
            is_verified,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Update last_login_at
    pub async fn update_last_login(&self, id: Uuid) -> DatabaseResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                last_login_at = now(),
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user.ok_or(DatabaseError::NotFound)
    }

    // Soft delete user
    pub async fn delete(&self, id: Uuid) -> DatabaseResult<PgQueryResult> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET
                deleted_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::NotFound);
        }

        Ok(result)
    }
}
