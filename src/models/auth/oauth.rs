use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthProvider {
    pub id: Uuid,
    pub provider_name: String,
    pub display_name: String,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub redirect_url: String,
    pub scope: String,
    pub is_active: bool,
    pub icon_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserOAuthConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_id: Uuid,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub raw_user_info: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct OAuthProviderResponse {
    pub id: Uuid,
    pub provider_name: String,
    pub display_name: String,
    pub auth_url: String,
    pub redirect_url: String,
    pub scope: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOAuthProviderDto {
    pub provider_name: String,
    pub display_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub redirect_url: String,
    pub scope: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOAuthProviderDto {
    pub display_name: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub auth_url: Option<String>,
    pub token_url: Option<String>,
    pub user_info_url: Option<String>,
    pub redirect_url: Option<String>,
    pub scope: Option<String>,
    pub is_active: Option<bool>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
    pub error: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthStartQuery {
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    pub url: String,
    pub state: String,
}

// Implementation of From trait for converting from OAuthProvider to OAuthProviderResponse
impl From<OAuthProvider> for OAuthProviderResponse {
    fn from(provider: OAuthProvider) -> Self {
        Self {
            id: provider.id,
            provider_name: provider.provider_name,
            display_name: provider.display_name,
            auth_url: provider.auth_url,
            redirect_url: provider.redirect_url,
            scope: provider.scope,
            icon_url: provider.icon_url,
        }
    }
}
