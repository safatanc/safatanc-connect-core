use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::db::error::DatabaseError;
use crate::db::repositories::TokenRepository;
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::auth::token::VerificationToken;
use crate::models::auth::token::{
    CreateVerificationTokenDto, TOKEN_TYPE_EMAIL_VERIFICATION, TOKEN_TYPE_PASSWORD_RESET,
};
use crate::models::user::{AuthResponse, CreateUserDto, LoginDto, User, UserResponse};
use crate::services::auth::token::TokenService;
use crate::services::user::user_management::UserManagementService;
use crate::services::validation::validation_err_to_app_error;

pub struct AuthService {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    token_service: Arc<TokenService>,
    user_management: Arc<UserManagementService>,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        token_repo: TokenRepository,
        token_service: Arc<TokenService>,
        user_management: Arc<UserManagementService>,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            token_service,
            user_management,
        }
    }

    // User login
    pub async fn login(&self, dto: &LoginDto) -> Result<AuthResponse, AppError> {
        // Validate DTO
        dto.validate().map_err(validation_err_to_app_error)?;

        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await
            .map_err(|_| AppError::Authentication("Invalid email or password".to_string()))?;

        // Verify password
        self.user_management
            .verify_password(&dto.password, &user.password_hash)
            .map_err(|_| AppError::Authentication("Invalid email or password".to_string()))?;

        // Generate token pair
        let token_pair = self.token_service.generate_tokens(&user)?;

        // Update last login timestamp
        let user = self
            .user_repo
            .update_last_login(user.id)
            .await
            .map_err(AppError::from)?;

        // Create response
        let response = AuthResponse {
            user: UserResponse::from(user),
            token: token_pair.0,
            refresh_token: token_pair.1,
        };

        Ok(response)
    }

    // Refresh token to get a new access token
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, AppError> {
        // Verify refresh token
        let claims = self.token_service.verify_token(refresh_token)?;

        // Ensure user still exists and is active
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid token".into()))?;

        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::Authentication("User not found".into()),
                _ => AppError::Database(e),
            })?;

        // Generate new token
        let new_token = self.token_service.refresh_token(refresh_token)?;

        Ok(new_token)
    }

    // Logout user (can be extended for token blacklisting, etc.)
    pub async fn logout(&self, user_id: Uuid) -> Result<(), AppError> {
        // For simple implementation, just verify that the user exists
        let _user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::NotFound("User not found".into()),
                _ => AppError::Database(e),
            })?;

        // Here you could add logic such as:
        // - Adding refresh token to blacklist
        // - Removing user session from database
        // - Logging logout activity, etc.

        Ok(())
    }

    // Verify token and get user ID
    pub fn verify_token(&self, token: &str) -> Result<Uuid, AppError> {
        self.token_service.get_user_id_from_token(token)
    }

    // Method for user registration with verification token creation
    pub async fn register_user_with_verification(
        &self,
        dto: CreateUserDto,
    ) -> Result<(User, VerificationToken), AppError> {
        // Validate DTO
        dto.validate().map_err(validation_err_to_app_error)?;

        // Hash password
        let password_hash = self.user_management.hash_password(&dto.password)?;

        // Create user
        let user = self
            .user_repo
            .create(&dto, password_hash)
            .await
            .map_err(|e| match e {
                DatabaseError::Duplicate(msg) => AppError::Validation(msg),
                _ => AppError::Database(e),
            })?;

        // Generate a verification token
        let token_string = self.generate_random_token(32)?;

        // Create the verification token
        let token_dto = CreateVerificationTokenDto {
            user_id: Some(user.id),
            token_type: TOKEN_TYPE_EMAIL_VERIFICATION.to_string(),
            expires_in: 86400, // 24 hours in seconds
        };

        // Use token repository to create the token
        let token = self
            .token_repo
            .create(&token_dto, &token_string)
            .await
            .map_err(AppError::Database)?;

        Ok((user, token))
    }

    // Simple random token generator
    fn generate_random_token(&self, length: usize) -> Result<String, AppError> {
        use rand::{distributions::Alphanumeric, Rng};

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        Ok(token)
    }

    // Verify email token and mark user's email as verified
    pub async fn verify_email_token(&self, token: &str) -> Result<UserResponse, AppError> {
        // Verify the token from the database
        let verification_token = self
            .token_repo
            .verify_token(token, TOKEN_TYPE_EMAIL_VERIFICATION)
            .await
            .map_err(|_| AppError::InvalidToken("Invalid or expired verification token".into()))?;

        // Ensure the token is linked to a user
        let user_id = verification_token
            .user_id
            .ok_or_else(|| AppError::InvalidToken("Token is not associated with a user".into()))?;

        // Mark the token as used
        self.token_repo
            .mark_as_used(verification_token.id)
            .await
            .map_err(AppError::Database)?;

        // Mark user's email as verified
        let user_response = self.user_management.verify_email(user_id).await?;

        Ok(user_response)
    }

    // Request password reset and generate token
    pub async fn request_password_reset(&self, email: &str) -> Result<VerificationToken, AppError> {
        // Validate email format
        crate::services::validation::validate_email(email)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Find user by email
        let user = self
            .user_repo
            .find_by_email(email)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::NotFound("User not found".into()),
                _ => AppError::Database(e),
            })?;

        // Invalidate any existing password reset tokens for this user
        let _ = self
            .token_repo
            .invalidate_by_user_and_type(user.id, TOKEN_TYPE_PASSWORD_RESET)
            .await;

        // Generate a new token
        let token_string = self.generate_random_token(32)?;

        // Create the password reset token
        let token_dto = CreateVerificationTokenDto {
            user_id: Some(user.id),
            token_type: TOKEN_TYPE_PASSWORD_RESET.to_string(),
            expires_in: 3600, // 1 hour in seconds
        };

        // Use token repository to create the token
        let token = self
            .token_repo
            .create(&token_dto, &token_string)
            .await
            .map_err(AppError::Database)?;

        // Here you would typically send an email with the reset link
        // For now, we're just returning the token for testing purposes
        // In production, don't return the token directly to the API

        Ok(token)
    }

    // Reset password using token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        // Validate password
        crate::services::validation::validate_password_strength(new_password)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Verify the token
        let verification_token = self
            .token_repo
            .verify_token(token, TOKEN_TYPE_PASSWORD_RESET)
            .await
            .map_err(|_| AppError::InvalidToken("Invalid or expired reset token".into()))?;

        // Ensure the token is linked to a user
        let user_id = verification_token
            .user_id
            .ok_or_else(|| AppError::InvalidToken("Token is not associated with a user".into()))?;

        // Hash the new password
        let password_hash = self.user_management.hash_password(new_password)?;

        // Update the user's password
        self.user_repo
            .update_password(user_id, &password_hash)
            .await
            .map_err(AppError::Database)?;

        // Mark the token as used
        self.token_repo
            .mark_as_used(verification_token.id)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    // OAuth Integrations

    // Get OAuth redirect URL
    pub async fn get_oauth_redirect_url(&self, provider: &str) -> Result<String, AppError> {
        // Create an OAuth client for the provider
        let client = self.create_oauth_client(provider)?;

        // Generate the authorization URL
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        // In a real application, you'd store the CSRF token in a session or cookie
        // For simplicity, we're not handling CSRF protection here

        Ok(auth_url.to_string())
    }

    // Handle OAuth callback
    pub async fn handle_oauth_callback(
        &self,
        provider: &str,
        code: &str,
    ) -> Result<AuthResponse, AppError> {
        // Create an OAuth client for the provider
        let client = self.create_oauth_client(provider)?;

        // Exchange the authorization code for an access token
        let token_result = client
            .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::Authentication(format!("Failed to exchange code: {}", e)))?;

        // Get the access token
        let access_token = token_result.access_token().secret();

        // Fetch user info from the provider using the access token
        let user_info = self.get_oauth_user_info(provider, access_token).await?;

        // Extract user details from the provider response
        let (email, name, avatar) = self.extract_user_info_from_response(provider, &user_info)?;

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
                    full_name: Some(name),
                    avatar_url: avatar,
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

    // Helper to create OAuth client for a provider
    fn create_oauth_client(&self, provider: &str) -> Result<BasicClient, AppError> {
        // In a real application, you'd fetch this configuration from a database
        // For simplicity, we're hardcoding supported providers here

        match provider.to_lowercase().as_str() {
            "google" => {
                let client_id = "your-google-client-id.apps.googleusercontent.com";
                let client_secret = "your-google-client-secret";
                let auth_url = "https://accounts.google.com/o/oauth2/v2/auth";
                let token_url = "https://oauth2.googleapis.com/token";
                let redirect_url = "http://localhost:3000/api/auth/oauth/google/callback";

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
                let redirect_url = "http://localhost:3000/api/auth/oauth/github/callback";

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

    // Fetch user info from the OAuth provider
    async fn get_oauth_user_info(
        &self,
        provider: &str,
        access_token: &str,
    ) -> Result<Value, AppError> {
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

        Ok(user_info)
    }

    // Extract user info from OAuth provider response
    fn extract_user_info_from_response(
        &self,
        provider: &str,
        response: &Value,
    ) -> Result<(String, String, Option<String>), AppError> {
        match provider.to_lowercase().as_str() {
            "google" => {
                let email = response["email"]
                    .as_str()
                    .ok_or_else(|| {
                        AppError::Authentication("Email not provided by OAuth provider".into())
                    })?
                    .to_string();

                let name = response["name"]
                    .as_str()
                    .unwrap_or("Google User")
                    .to_string();

                let avatar = response["picture"].as_str().map(|s| s.to_string());

                Ok((email, name, avatar))
            }
            "github" => {
                let email = match response["email"].as_str() {
                    Some(email) => email.to_string(),
                    None => {
                        // GitHub might not provide email in the initial response
                        // In a real app, you'd make a separate request to fetch emails
                        // For simplicity, we'll generate a placeholder email
                        let username = response["login"].as_str().ok_or_else(|| {
                            AppError::Authentication("Username not provided by GitHub".into())
                        })?;
                        format!("{}@github.user", username)
                    }
                };

                let name = response["name"]
                    .as_str()
                    .unwrap_or_else(|| response["login"].as_str().unwrap_or("GitHub User"))
                    .to_string();

                let avatar = response["avatar_url"].as_str().map(|s| s.to_string());

                Ok((email, name, avatar))
            }
            _ => Err(AppError::Validation(format!(
                "Unsupported OAuth provider: {}",
                provider
            ))),
        }
    }
}
