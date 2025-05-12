use chrono::NaiveDateTime;
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::auth::oauth::{
    CreateOAuthProviderDto, OAuthProvider, UpdateOAuthProviderDto, UserOAuthConnection,
};

#[derive(Clone)]
pub struct OAuthRepository {
    pool: PgPool,
}

impl OAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // *** OAuth Provider Methods ***

    // Create a new OAuth provider
    pub async fn create_provider(
        &self,
        dto: &CreateOAuthProviderDto,
    ) -> DatabaseResult<OAuthProvider> {
        sqlx::query_as!(
            OAuthProvider,
            r#"
            INSERT INTO oauth_providers (
                provider_name, display_name, client_id, client_secret, auth_url, 
                token_url, user_info_url, redirect_url, scope, icon_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            "#,
            dto.provider_name,
            dto.display_name,
            dto.client_id,
            dto.client_secret,
            dto.auth_url,
            dto.token_url,
            dto.user_info_url,
            dto.redirect_url,
            dto.scope,
            dto.icon_url
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "oauth_providers_provider_name_key" => {
                            DatabaseError::Duplicate("Provider name already exists".to_string())
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

    // Find OAuth provider by ID
    pub async fn find_provider_by_id(&self, id: Uuid) -> DatabaseResult<OAuthProvider> {
        let provider = sqlx::query_as!(
            OAuthProvider,
            r#"
            SELECT 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            FROM oauth_providers
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        provider.ok_or(DatabaseError::NotFound)
    }

    // Find OAuth provider by name
    pub async fn find_provider_by_name(
        &self,
        provider_name: &str,
    ) -> DatabaseResult<OAuthProvider> {
        let provider = sqlx::query_as!(
            OAuthProvider,
            r#"
            SELECT 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            FROM oauth_providers
            WHERE provider_name = $1 AND deleted_at IS NULL
            "#,
            provider_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        provider.ok_or(DatabaseError::NotFound)
    }

    // Get all active OAuth providers
    pub async fn find_all_providers(&self) -> DatabaseResult<Vec<OAuthProvider>> {
        let providers = sqlx::query_as!(
            OAuthProvider,
            r#"
            SELECT 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            FROM oauth_providers
            WHERE deleted_at IS NULL
            ORDER BY display_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(providers)
    }

    // Update OAuth provider
    pub async fn update_provider(
        &self,
        id: Uuid,
        dto: &UpdateOAuthProviderDto,
    ) -> DatabaseResult<OAuthProvider> {
        let provider = sqlx::query_as!(
            OAuthProvider,
            r#"
            UPDATE oauth_providers
            SET
                display_name = COALESCE($1, display_name),
                client_id = COALESCE($2, client_id),
                client_secret = COALESCE($3, client_secret),
                auth_url = COALESCE($4, auth_url),
                token_url = COALESCE($5, token_url),
                user_info_url = COALESCE($6, user_info_url),
                redirect_url = COALESCE($7, redirect_url),
                scope = COALESCE($8, scope),
                is_active = COALESCE($9, is_active),
                icon_url = $10,
                updated_at = NOW()
            WHERE id = $11 AND deleted_at IS NULL
            RETURNING 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            "#,
            dto.display_name,
            dto.client_id,
            dto.client_secret,
            dto.auth_url,
            dto.token_url,
            dto.user_info_url,
            dto.redirect_url,
            dto.scope,
            dto.is_active,
            dto.icon_url,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        provider.ok_or(DatabaseError::NotFound)
    }

    // Soft delete an OAuth provider
    pub async fn delete_provider(&self, id: Uuid) -> DatabaseResult<OAuthProvider> {
        let provider = sqlx::query_as!(
            OAuthProvider,
            r#"
            UPDATE oauth_providers
            SET
                deleted_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING 
                id, provider_name, display_name, client_id, client_secret, 
                auth_url, token_url, user_info_url, redirect_url, scope, 
                is_active, icon_url, created_at, updated_at, deleted_at
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        provider.ok_or(DatabaseError::NotFound)
    }

    // *** User OAuth Connection Methods ***

    // Create or update a user OAuth connection
    pub async fn upsert_connection(
        &self,
        user_id: Uuid,
        provider_id: Uuid,
        provider_user_id: &str,
        email: Option<&str>,
        name: Option<&str>,
        avatar_url: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        expires_at: Option<NaiveDateTime>,
        raw_user_info: Option<serde_json::Value>,
    ) -> DatabaseResult<UserOAuthConnection> {
        sqlx::query_as!(
            UserOAuthConnection,
            r#"
            INSERT INTO user_oauth_connections (
                user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id, provider_id) DO UPDATE
            SET
                provider_user_id = $3,
                email = COALESCE($4, user_oauth_connections.email),
                name = COALESCE($5, user_oauth_connections.name),
                avatar_url = COALESCE($6, user_oauth_connections.avatar_url),
                access_token = COALESCE($7, user_oauth_connections.access_token),
                refresh_token = COALESCE($8, user_oauth_connections.refresh_token),
                expires_at = COALESCE($9, user_oauth_connections.expires_at),
                raw_user_info = COALESCE($10, user_oauth_connections.raw_user_info),
                updated_at = NOW()
            RETURNING 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            "#,
            user_id,
            provider_id,
            provider_user_id,
            email,
            name,
            avatar_url,
            access_token,
            refresh_token,
            expires_at,
            raw_user_info
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    // Find user OAuth connection by ID
    pub async fn find_connection_by_id(&self, id: Uuid) -> DatabaseResult<UserOAuthConnection> {
        let connection = sqlx::query_as!(
            UserOAuthConnection,
            r#"
            SELECT 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            FROM user_oauth_connections
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        connection.ok_or(DatabaseError::NotFound)
    }

    // Find user OAuth connection by provider and provider user ID
    pub async fn find_connection_by_provider_user_id(
        &self,
        provider_id: Uuid,
        provider_user_id: &str,
    ) -> DatabaseResult<UserOAuthConnection> {
        let connection = sqlx::query_as!(
            UserOAuthConnection,
            r#"
            SELECT 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            FROM user_oauth_connections
            WHERE provider_id = $1 AND provider_user_id = $2 AND deleted_at IS NULL
            "#,
            provider_id,
            provider_user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        connection.ok_or(DatabaseError::NotFound)
    }

    // Find user OAuth connections by user ID
    pub async fn find_connections_by_user_id(
        &self,
        user_id: Uuid,
    ) -> DatabaseResult<Vec<UserOAuthConnection>> {
        let connections = sqlx::query_as!(
            UserOAuthConnection,
            r#"
            SELECT 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            FROM user_oauth_connections
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(connections)
    }

    // Find user OAuth connection by user ID and provider ID
    pub async fn find_connection_by_user_and_provider(
        &self,
        user_id: Uuid,
        provider_id: Uuid,
    ) -> DatabaseResult<UserOAuthConnection> {
        let connection = sqlx::query_as!(
            UserOAuthConnection,
            r#"
            SELECT 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            FROM user_oauth_connections
            WHERE user_id = $1 AND provider_id = $2 AND deleted_at IS NULL
            "#,
            user_id,
            provider_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        connection.ok_or(DatabaseError::NotFound)
    }

    // Delete user OAuth connection (soft delete)
    pub async fn delete_connection(&self, id: Uuid) -> DatabaseResult<UserOAuthConnection> {
        let connection = sqlx::query_as!(
            UserOAuthConnection,
            r#"
            UPDATE user_oauth_connections
            SET
                deleted_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING 
                id, user_id, provider_id, provider_user_id, email, name, 
                avatar_url, access_token, refresh_token, expires_at, raw_user_info,
                created_at, updated_at, deleted_at
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        connection.ok_or(DatabaseError::NotFound)
    }

    // Delete all OAuth connections for a user (for account deletion)
    pub async fn delete_all_user_connections(
        &self,
        user_id: Uuid,
    ) -> DatabaseResult<PgQueryResult> {
        sqlx::query!(
            r#"
            UPDATE user_oauth_connections
            SET
                deleted_at = NOW(),
                updated_at = NOW()
            WHERE user_id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }
}
