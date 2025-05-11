use std::sync::Arc;
use uuid::Uuid;

use crate::db::error::DatabaseError;
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::user::{AuthResponse, LoginDto, UserResponse, CreateUserDto, User, VerificationToken};
use crate::services::auth::token::TokenService;
use crate::services::user::user_management::UserManagementService;
use crate::db::executor::run_transaction;
use sqlx::Transaction;
use crate::models::auth::token::{CreateVerificationTokenDto, TOKEN_TYPE_EMAIL_VERIFICATION};
use crate::db::repositories::TokenRepository;
use sqlx::PgPool;

pub struct AuthService {
    user_repo: Arc<UserRepository<PgPool>>,
    token_service: Arc<TokenService>,
    user_management: Arc<UserManagementService>,
    pool: Arc<PgPool>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<UserRepository<PgPool>>,
        token_service: Arc<TokenService>,
        user_management: Arc<UserManagementService>,
        pool: Arc<PgPool>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
            user_management,
            pool,
        }
    }

    // User login
    pub async fn login(&self, dto: &LoginDto) -> Result<AuthResponse, AppError> {
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
        let token_pair = self.token_service.generate_token_pair(user.id)?;

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

    // Example method using a transaction for user registration
    // and verification token creation in a single transaction
    pub async fn register_user_with_verification(
        &self, 
        dto: CreateUserDto
    ) -> Result<(User, VerificationToken), AppError> {
        // Use run_transaction to run multiple operations in a single transaction
        run_transaction(self.pool.as_ref(), |tx| {
            let dto = dto.clone(); // Clone dto to use in the closure
            
            Box::new(async move {
                // Hash password
                let password_hash = self.user_management.hash_password(&dto.password)?;
                
                // Create repository that uses the transaction
                let tx_user_repo = UserRepository::new(Arc::new(tx.clone()));
                
                // Create user
                let user = tx_user_repo.create(&dto, password_hash).await?;
                
                // Create email verification token
                let token_dto = CreateVerificationTokenDto {
                    user_id: Some(user.id),
                    token_type: TOKEN_TYPE_EMAIL_VERIFICATION.to_string(),
                    expires_in: 86400, // 24 hours
                };
                
                // Create verification token within the transaction
                let token_string = self.token_service.generate_token(32)?;
                let tx_token_repo = TokenRepository::new(Arc::new(tx.clone()));
                let token = tx_token_repo.create(&token_dto, &token_string).await?;
                
                Ok::<(User, VerificationToken), AppError>((user, token))
            })
        }).await
    }
}
