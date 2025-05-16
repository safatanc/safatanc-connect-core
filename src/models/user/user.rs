use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::services::validation::{validate_email, validate_password_strength, validate_username};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub global_role: String,
    pub is_email_verified: bool,
    pub is_active: bool,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

pub const GLOBAL_ROLE_ADMIN: &str = "ADMIN";
pub const GLOBAL_ROLE_USER: &str = "USER";

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(custom = "validate_email")]
    pub email: String,

    #[validate(custom = "validate_username")]
    pub username: String,

    #[validate(custom = "validate_password_strength")]
    pub password: String,

    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(custom = "validate_username")]
    pub username: String,

    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(custom = "validate_email")]
    pub email: String,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordResetRequestDto {
    #[validate(custom = "validate_email")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendVerificationEmailDto {
    #[validate(custom = "validate_email")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordResetDto {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(custom = "validate_password_strength")]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePasswordDto {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(custom = "validate_password_strength")]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub global_role: String,
    pub is_email_verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
    pub refresh_token: String,
}

// Implementation of From trait for converting from User to UserResponse
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            global_role: user.global_role,
            is_email_verified: user.is_email_verified,
            created_at: user.created_at,
        }
    }
}
