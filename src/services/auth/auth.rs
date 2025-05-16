use std::sync::Arc;
use tokio::task;
use uuid::Uuid;
use validator::Validate;

use crate::db::error::DatabaseError;
use crate::db::repositories::TokenRepository;
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::auth::token::{
    CreateVerificationTokenDto, TOKEN_TYPE_EMAIL_VERIFICATION, TOKEN_TYPE_PASSWORD_RESET,
};
use crate::models::user::{AuthResponse, LoginDto, UserResponse};
use crate::services::auth::oauth::OAuthService;
use crate::services::auth::token::TokenService;
use crate::services::user::UserManagementService;
use crate::services::validation::validation_err_to_app_error;

pub struct AuthService {
    user_repo: UserRepository,
    token_repo: TokenRepository,
    token_service: Arc<TokenService>,
    user_management: Arc<UserManagementService>,
    oauth_service: Option<Arc<OAuthService>>,
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
            oauth_service: None,
        }
    }

    // Set OAuth service
    pub fn with_oauth_service(mut self, oauth_service: Arc<OAuthService>) -> Self {
        self.oauth_service = Some(oauth_service);
        self
    }

    // Login with username/email and password
    pub async fn login(&self, credentials: &LoginDto) -> Result<AuthResponse, AppError> {
        // Validate login data
        credentials
            .validate()
            .map_err(validation_err_to_app_error)?;

        // Get user by email or username
        let user = if credentials.email.contains('@') {
            self.user_repo
                .find_by_email(&credentials.email)
                .await
                .map_err(|e| match e {
                    DatabaseError::NotFound => {
                        AppError::Authentication("Invalid credentials".into())
                    }
                    _ => AppError::Database(e),
                })?
        } else {
            self.user_repo
                .find_by_username(&credentials.email)
                .await
                .map_err(|e| match e {
                    DatabaseError::NotFound => {
                        AppError::Authentication("Invalid credentials".into())
                    }
                    _ => AppError::Database(e),
                })?
        };

        // Verify password
        self.user_management
            .verify_password(&credentials.password, &user.password_hash)?;

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Authentication(
                "Account is disabled. Please contact support.".into(),
            ));
        }

        // Clone user for the response
        let response_user = user.clone();

        // Generate tokens
        let (token, refresh_token) = self.token_service.generate_tokens(&response_user)?;

        // Update last login timestamp asynchronously
        let user_repo = self.user_repo.clone();
        let user_id = user.id;
        tokio::spawn(async move {
            if let Err(e) = user_repo.update_last_login(user_id).await {
                tracing::error!("Failed to update last login timestamp: {}", e);
            }
        });

        // Create response
        let auth_response = AuthResponse {
            user: UserResponse::from(response_user),
            token,
            refresh_token,
        };

        Ok(auth_response)
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

    // Email verification
    pub async fn verify_email_token(&self, token: &str) -> Result<UserResponse, AppError> {
        // Verify the token
        let verification_token = self
            .token_repo
            .verify_token(token, TOKEN_TYPE_EMAIL_VERIFICATION)
            .await
            .map_err(|_| AppError::InvalidToken("Invalid or expired verification token".into()))?;

        // Ensure the token is linked to a user
        let user_id = verification_token
            .user_id
            .ok_or_else(|| AppError::InvalidToken("Token is not associated with a user".into()))?;

        // Get the user data first
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| match e {
                DatabaseError::NotFound => AppError::NotFound("User not found".into()),
                _ => AppError::Database(e),
            })?;

        // Check if already verified to avoid unnecessary updates
        if !user.is_email_verified {
            // Update the user's email verification status synchronously
            let updated_user = self
                .user_repo
                .update_email_verification(user_id, true)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to update email verification status: {}", e);
                    AppError::Database(e)
                })?;

            // Mark the token as used synchronously
            self.token_repo
                .mark_as_used(verification_token.id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to mark token as used: {}", e);
                    AppError::Database(e)
                })?;

            // Return updated user
            return Ok(UserResponse::from(updated_user));
        }

        // If already verified, just return the user
        Ok(UserResponse::from(user))
    }

    // Password reset request
    pub async fn request_password_reset(&self, email: &str) -> Result<String, AppError> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(email)
            .await
            .map_err(|e| match e {
                // Don't reveal if the email exists for security reasons
                DatabaseError::NotFound => {
                    AppError::NotFound("If the email exists, a reset link will be sent".into())
                }
                _ => AppError::Database(e),
            })?;

        // Generate a random token
        let token_string = self.generate_random_token(32)?;

        // Create a password reset token
        let token_dto = CreateVerificationTokenDto {
            user_id: Some(user.id),
            token_type: TOKEN_TYPE_PASSWORD_RESET.to_string(),
            expires_in: 24 * 60 * 60, // 24 hours in seconds
        };

        // Create token in database
        let token = self
            .token_repo
            .create(&token_dto, &token_string)
            .await
            .map_err(AppError::Database)?;

        // In a real application, you would send an email with the reset link
        // Return the token for demo purposes
        Ok(token.token)
    }

    // Reset password
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

        // Update password and mark token as used asynchronously
        let user_repo = self.user_repo.clone();
        let token_repo = self.token_repo.clone();
        let token_id = verification_token.id;

        // Spawn task to handle database updates
        task::spawn(async move {
            // Update the user's password
            if let Err(e) = user_repo.update_password(user_id, &password_hash).await {
                tracing::error!("Failed to update password: {}", e);
            }

            // Mark the token as used
            if let Err(e) = token_repo.mark_as_used(token_id).await {
                tracing::error!("Failed to mark token as used: {}", e);
            }
        });

        Ok(())
    }

    // OAuth redirect to use the new OAuthService
    pub async fn get_oauth_redirect_url(&self, provider: &str) -> Result<String, AppError> {
        match &self.oauth_service {
            Some(oauth_service) => oauth_service.get_oauth_redirect_url(provider).await,
            None => Err(AppError::Configuration(
                "OAuth service not configured".into(),
            )),
        }
    }

    // OAuth callback to use the new OAuthService
    pub async fn handle_oauth_callback(
        &self,
        provider: &str,
        code: &str,
    ) -> Result<AuthResponse, AppError> {
        match &self.oauth_service {
            Some(oauth_service) => oauth_service.handle_oauth_callback(provider, code).await,
            None => Err(AppError::Configuration(
                "OAuth service not configured".into(),
            )),
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
