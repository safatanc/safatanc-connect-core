use std::sync::Arc;
use uuid::Uuid;

use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::db::error::DatabaseError;
use crate::db::repositories::{OAuthRepository, UserRepository};
use crate::errors::AppError;
use crate::models::auth::oauth::{OAuthProvider, OAuthUrlResponse};
use crate::models::user::{AuthResponse, CreateUserDto, User, UserResponse};
use crate::services::auth::token::TokenService;
use crate::services::user::UserManagementService;

pub struct OAuthService {
    user_repo: UserRepository,
    oauth_repo: OAuthRepository,
    token_service: Arc<TokenService>,
    user_management: Arc<UserManagementService>,
}

impl OAuthService {
    pub fn new(
        user_repo: UserRepository,
        oauth_repo: OAuthRepository,
        token_service: Arc<TokenService>,
        user_management: Arc<UserManagementService>,
    ) -> Self {
        Self {
            user_repo,
            oauth_repo,
            token_service,
            user_management,
        }
    }

    // Get OAuth redirect URL
    pub async fn get_oauth_redirect_url(&self, provider: &str) -> Result<String, AppError> {
        // Try to get provider from database
        let provider_result = self.oauth_repo.find_provider_by_name(provider).await;

        match provider_result {
            Ok(provider_config) => {
                // Create an OAuth client with the stored configuration
                let client = self.create_oauth_client_from_config(&provider_config)?;

                // Generate the authorization URL
                let (auth_url, _csrf_token) = client
                    .authorize_url(CsrfToken::new_random)
                    .add_scope(Scope::new(provider_config.scope))
                    .url();

                // In a real application, you'd store the CSRF token in a session or cookie
                // For simplicity, we're not handling CSRF protection here

                Ok(auth_url.to_string())
            }
            Err(_) => {
                // Fall back to hardcoded configuration
                self.create_oauth_redirect_url_fallback(provider)
            }
        }
    }

    // Handle OAuth callback
    pub async fn handle_oauth_callback(
        &self,
        provider: &str,
        code: &str,
    ) -> Result<AuthResponse, AppError> {
        // Get provider from database or use fallback
        let oauth_client = match self.oauth_repo.find_provider_by_name(provider).await {
            Ok(provider_config) => self.create_oauth_client_from_config(&provider_config)?,
            Err(_) => self.create_oauth_client_fallback(provider)?,
        };

        // Exchange the authorization code for an access token
        let token_result = oauth_client
            .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::Authentication(format!("Failed to exchange code: {}", e)))?;

        // Get the access token
        let access_token = token_result.access_token().secret();

        // Fetch user info from the provider using the access token
        let (provider_user_id, email, name, avatar) =
            match self.oauth_repo.find_provider_by_name(provider).await {
                Ok(provider_config) => {
                    self.get_oauth_user_info_from_config(&provider_config, access_token)
                        .await?
                }
                Err(_) => {
                    self.get_oauth_user_info_fallback(provider, access_token)
                        .await?
                }
            };

        // Check if user exists with this email
        let user = match self.user_repo.find_by_email(&email).await {
            Ok(user) => {
                // User exists, update their last login
                self.user_repo
                    .update_last_login(user.id)
                    .await
                    .map_err(AppError::from)?
            }
            Err(DatabaseError::NotFound) => {
                // Create a new user
                let mut create_user_dto = CreateUserDto {
                    email: email.clone(),
                    username: email.split('@').next().unwrap_or("user").to_string(),
                    password: self.generate_random_token(32)?, // Random password
                    full_name: Some(name.clone()),
                    avatar_url: avatar.clone(),
                };

                // Ensure username is unique by adding random characters if needed
                let username_base = create_user_dto.username.clone();
                let mut attempt = 0;

                while self
                    .user_repo
                    .find_by_username(&create_user_dto.username)
                    .await
                    .is_ok()
                {
                    attempt += 1;
                    create_user_dto.username = format!("{}_{}", username_base.clone(), attempt);
                }

                // Hash the random password
                let password_hash = self
                    .user_management
                    .hash_password(&create_user_dto.password)?;

                // Create the user with email verified since it came from OAuth
                let mut user = self
                    .user_repo
                    .create(&create_user_dto, password_hash)
                    .await
                    .map_err(|e| match e {
                        DatabaseError::Duplicate(msg) => AppError::Validation(msg),
                        _ => AppError::Database(e),
                    })?;

                // Mark email as verified since it's from the OAuth provider
                user = self
                    .user_repo
                    .update_email_verification(user.id, true)
                    .await
                    .map_err(AppError::Database)?;

                user
            }
            Err(e) => return Err(AppError::Database(e)),
        };

        // Store the OAuth connection if provider was found in database
        if let Ok(provider_config) = self.oauth_repo.find_provider_by_name(provider).await {
            let refresh_token = token_result.refresh_token().map(|rt| rt.secret().clone());
            let expires_in = token_result.expires_in().map(|d| {
                let now = chrono::Utc::now();
                let expiry = now + d;
                expiry.naive_utc()
            });

            // Store or update the OAuth connection
            self.oauth_repo
                .upsert_connection(
                    user.id,
                    provider_config.id,
                    &provider_user_id,
                    Some(&email),
                    Some(&name),
                    avatar.as_deref(),
                    Some(access_token),
                    refresh_token.as_deref(),
                    expires_in,
                    None, // We could store the raw user info here
                )
                .await
                .map_err(AppError::Database)?;
        }

        // Generate JWT tokens
        let token_pair = self.token_service.generate_tokens(&user)?;

        // Create auth response
        let auth_response = AuthResponse {
            user: UserResponse::from(user),
            token: token_pair.0,
            refresh_token: token_pair.1,
        };

        Ok(auth_response)
    }

    // Helper function to create OAuth client from database configuration
    fn create_oauth_client_from_config(
        &self,
        provider: &OAuthProvider,
    ) -> Result<BasicClient, AppError> {
        Ok(BasicClient::new(
            ClientId::new(provider.client_id.clone()),
            Some(ClientSecret::new(provider.client_secret.clone())),
            AuthUrl::new(provider.auth_url.clone())
                .map_err(|_| AppError::Configuration("Invalid auth URL".to_string()))?,
            Some(
                TokenUrl::new(provider.token_url.clone())
                    .map_err(|_| AppError::Configuration("Invalid token URL".to_string()))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(provider.redirect_url.clone())
                .map_err(|_| AppError::Configuration("Invalid redirect URL".to_string()))?,
        ))
    }

    // Fallback methods for hardcoded OAuth provider configurations
    fn create_oauth_client_fallback(&self, provider: &str) -> Result<BasicClient, AppError> {
        match provider.to_lowercase().as_str() {
            "google" => {
                let client_id = "your-google-client-id.apps.googleusercontent.com";
                let client_secret = "your-google-client-secret";
                let auth_url = "https://accounts.google.com/o/oauth2/v2/auth";
                let token_url = "https://oauth2.googleapis.com/token";
                let redirect_url = "http://localhost:8080/api/auth/oauth/google/callback";

                Ok(BasicClient::new(
                    ClientId::new(client_id.to_string()),
                    Some(ClientSecret::new(client_secret.to_string())),
                    AuthUrl::new(auth_url.to_string()).unwrap(),
                    Some(TokenUrl::new(token_url.to_string()).unwrap()),
                )
                .set_redirect_uri(RedirectUrl::new(redirect_url.to_string()).unwrap()))
            }
            "github" => {
                let client_id = "your-github-client-id";
                let client_secret = "your-github-client-secret";
                let auth_url = "https://github.com/login/oauth/authorize";
                let token_url = "https://github.com/login/oauth/access_token";
                let redirect_url = "http://localhost:8080/api/auth/oauth/github/callback";

                Ok(BasicClient::new(
                    ClientId::new(client_id.to_string()),
                    Some(ClientSecret::new(client_secret.to_string())),
                    AuthUrl::new(auth_url.to_string()).unwrap(),
                    Some(TokenUrl::new(token_url.to_string()).unwrap()),
                )
                .set_redirect_uri(RedirectUrl::new(redirect_url.to_string()).unwrap()))
            }
            _ => Err(AppError::Validation(format!(
                "Unsupported OAuth provider: {}",
                provider
            ))),
        }
    }

    fn create_oauth_redirect_url_fallback(&self, provider: &str) -> Result<String, AppError> {
        let client = self.create_oauth_client_fallback(provider)?;

        // Generate the authorization URL
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        Ok(auth_url.to_string())
    }

    // Get user info from OAuth provider
    async fn get_oauth_user_info_from_config(
        &self,
        provider: &OAuthProvider,
        access_token: &str,
    ) -> Result<(String, String, String, Option<String>), AppError> {
        let client = HttpClient::new();

        // Make the request to the user info endpoint
        let response = client
            .get(&provider.user_info_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .header("User-Agent", "SafaTanc-Connect")
            .send()
            .await
            .map_err(|e| AppError::Unexpected(format!("Failed to fetch user info: {}", e)))?;

        // Parse the response
        let user_info: Value = response
            .json()
            .await
            .map_err(|e| AppError::Unexpected(format!("Failed to parse user info: {}", e)))?;

        // Extract info based on provider
        match provider.provider_name.to_lowercase().as_str() {
            "google" => {
                let provider_user_id = user_info["id"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::Authentication("User ID not provided by OAuth provider".into())
                    })?
                    .to_string();

                let email = user_info["email"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::Authentication("Email not provided by OAuth provider".into())
                    })?
                    .to_string();

                let name = user_info["name"]
                    .as_str()
                    .unwrap_or("Google User")
                    .to_string();

                let avatar = user_info["picture"].as_str().map(|s| s.to_string());

                Ok((provider_user_id, email, name, avatar))
            }
            "github" => {
                let provider_user_id = match user_info["id"].as_str() {
                    Some(id) => id.to_string(),
                    None => user_info["id"].to_string().trim_matches('"').to_string(),
                };

                let email = match user_info["email"].as_str() {
                    Some(email) => email.to_string(),
                    None => {
                        // GitHub might not provide email in the initial response
                        // In a real app, you'd make a separate request to fetch emails
                        // For simplicity, we'll generate a placeholder email
                        let username = user_info["login"].as_str().ok_or_else(|| {
                            AppError::Authentication("Username not provided by GitHub".into())
                        })?;
                        format!("{}@github.user", username)
                    }
                };

                let name = user_info["name"]
                    .as_str()
                    .unwrap_or_else(|| user_info["login"].as_str().unwrap_or("GitHub User"))
                    .to_string();

                let avatar = user_info["avatar_url"].as_str().map(|s| s.to_string());

                Ok((provider_user_id, email, name, avatar))
            }
            _ => Err(AppError::Validation(format!(
                "Unsupported OAuth provider: {}",
                provider.provider_name
            ))),
        }
    }

    // Fallback method for hardcoded providers
    async fn get_oauth_user_info_fallback(
        &self,
        provider: &str,
        access_token: &str,
    ) -> Result<(String, String, String, Option<String>), AppError> {
        let client = HttpClient::new();
        let url = match provider.to_lowercase().as_str() {
            "google" => "https://www.googleapis.com/oauth2/v2/userinfo",
            "github" => "https://api.github.com/user",
            _ => {
                return Err(AppError::Validation(format!(
                    "Unsupported OAuth provider: {}",
                    provider
                )))
            }
        };

        let mut req = client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token));

        // Add special headers for GitHub
        if provider.to_lowercase() == "github" {
            req = req
                .header("Accept", "application/json")
                .header("User-Agent", "SafaTanc-Connect");
        }

        // Make the request
        let response = req
            .send()
            .await
            .map_err(|e| AppError::Unexpected(format!("Failed to fetch user info: {}", e)))?;

        // Parse the response
        let user_info: Value = response
            .json()
            .await
            .map_err(|e| AppError::Unexpected(format!("Failed to parse user info: {}", e)))?;

        match provider.to_lowercase().as_str() {
            "google" => {
                let provider_user_id = user_info["id"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::Authentication("User ID not provided by OAuth provider".into())
                    })?
                    .to_string();

                let email = user_info["email"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::Authentication("Email not provided by OAuth provider".into())
                    })?
                    .to_string();

                let name = user_info["name"]
                    .as_str()
                    .unwrap_or("Google User")
                    .to_string();

                let avatar = user_info["picture"].as_str().map(|s| s.to_string());

                Ok((provider_user_id, email, name, avatar))
            }
            "github" => {
                let provider_user_id = user_info["id"].to_string().trim_matches('"').to_string();

                let email = match user_info["email"].as_str() {
                    Some(email) => email.to_string(),
                    None => {
                        // GitHub might not provide email in the initial response
                        // In a real app, you'd make a separate request to fetch emails
                        // For simplicity, we'll generate a placeholder email
                        let username = user_info["login"].as_str().ok_or_else(|| {
                            AppError::Authentication("Username not provided by GitHub".into())
                        })?;
                        format!("{}@github.user", username)
                    }
                };

                let name = user_info["name"]
                    .as_str()
                    .unwrap_or_else(|| user_info["login"].as_str().unwrap_or("GitHub User"))
                    .to_string();

                let avatar = user_info["avatar_url"].as_str().map(|s| s.to_string());

                Ok((provider_user_id, email, name, avatar))
            }
            _ => Err(AppError::Validation(format!(
                "Unsupported OAuth provider: {}",
                provider
            ))),
        }
    }

    // Helper to generate random token
    fn generate_random_token(&self, length: usize) -> Result<String, AppError> {
        use rand::{distributions::Alphanumeric, Rng};

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        Ok(token)
    }
}
