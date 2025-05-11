use std::sync::Arc;
use uuid::Uuid;

use crate::db::error::DatabaseError;
use crate::db::repositories::TokenRepository;
use crate::db::repositories::UserRepository;
use crate::errors::AppError;
use crate::models::auth::token::VerificationToken;
use crate::models::auth::token::{CreateVerificationTokenDto, TOKEN_TYPE_EMAIL_VERIFICATION};
use crate::models::user::{AuthResponse, CreateUserDto, LoginDto, User, UserResponse};
use crate::services::auth::token::TokenService;
use crate::services::user::user_management::UserManagementService;
use sqlx::PgPool;

pub struct AuthService {
    user_repo: UserRepository,
    token_service: Arc<TokenService>,
    user_management: Arc<UserManagementService>,
    pool: PgPool,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        token_service: Arc<TokenService>,
        user_management: Arc<UserManagementService>,
        pool: PgPool,
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

    // Example method using a transaction for user registration
    // and verification token creation in a single transaction
    pub async fn register_user_with_verification(
        &self,
        dto: CreateUserDto,
    ) -> Result<(User, VerificationToken), AppError> {
        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(AppError::from)?;

        // Hash password
        let password_hash = self.user_management.hash_password(&dto.password)?;

        // Create user directly with the transaction
        let user = sqlx::query_as!(
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
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "users_email_key" => {
                            AppError::Validation("Email already exists".to_string())
                        }
                        "users_username_key" => {
                            AppError::Validation("Username already exists".to_string())
                        }
                        _ => AppError::Database(DatabaseError::ConnectionError(e)),
                    }
                } else {
                    AppError::Database(DatabaseError::ConnectionError(e))
                }
            } else {
                AppError::Database(DatabaseError::ConnectionError(e))
            }
        })?;

        // Generate a verification token
        let token_string = self.generate_random_token(32)?;

        // Create verification token within the transaction
        let token = sqlx::query_as!(
            VerificationToken,
            r#"
            INSERT INTO verification_tokens (
                user_id, token, token_type, expires_at
            )
            VALUES ($1, $2, $3, NOW() + $4 * INTERVAL '1 second')
            RETURNING 
                id, user_id, token, token_type, expires_at, used_at,
                created_at, updated_at
            "#,
            user.id,
            token_string,
            TOKEN_TYPE_EMAIL_VERIFICATION,
            86400i32, // 24 hours
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(DatabaseError::ConnectionError(e)))?;

        // Commit the transaction
        tx.commit().await.map_err(AppError::from)?;

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
}
