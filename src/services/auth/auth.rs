use std::sync::Arc;
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

        // Check if email is verified (optional, depends on your requirements)
        if !user.is_email_verified {
            // You might want to allow login but with limited functionality, or require verification
            // For this example, we'll log the user in but note they need to verify email
            // In a real app, you might trigger email verification here
        }

        // Update last login timestamp
        let updated_user = self
            .user_repo
            .update_last_login(user.id)
            .await
            .map_err(AppError::Database)?;

        // Generate tokens
        let (token, refresh_token) = self.token_service.generate_tokens(&updated_user)?;

        // Create response
        let auth_response = AuthResponse {
            user: UserResponse::from(updated_user),
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

        // Update the user's email verification status
        let user = self
            .user_repo
            .update_email_verification(user_id, true)
            .await
            .map_err(AppError::Database)?;

        // Mark the token as used
        self.token_repo
            .mark_as_used(verification_token.id)
            .await
            .map_err(AppError::Database)?;

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
