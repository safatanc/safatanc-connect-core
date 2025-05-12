use std::sync::Arc;

use axum::extract::Extension;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::routes::AuthApiState;
use crate::db::repositories::Repositories;
use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::auth::oauth::OAuthCallbackQuery;
use crate::models::common::response::ApiResponse;
use crate::models::user::{CreateUserDto, LoginDto, PasswordResetDto, UserResponse};
use crate::services::validation::validation_err_to_app_error;

// Login handler
pub async fn login(
    Json(credentials): Json<LoginDto>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Validate credentials
    credentials
        .validate()
        .map_err(validation_err_to_app_error)?;

    // Call auth service to login
    let response = state.auth_service.login(&credentials).await?;

    Ok(ApiResponse::success(StatusCode::OK, response))
}

// Register handler
pub async fn register(
    State(state): State<Arc<AuthApiState>>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Response, AppError> {
    // Validate registration data
    dto.validate().map_err(validation_err_to_app_error)?;

    // Register the user
    let user = state.user_management_service.register_user(dto).await?;

    // Return registered user data
    Ok(ApiResponse::created(UserResponse::from(user)))
}

// Refresh token handler
pub async fn refresh_token(
    State(state): State<Arc<AuthApiState>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    // Extract refresh token from request
    let refresh_token = data
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Refresh token is required".to_string()))?;

    // Call token service to refresh
    let new_token = state.token_service.refresh_token(refresh_token)?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        serde_json::json!({ "token": new_token }),
    ))
}

// Logout handler
pub async fn logout(
    State(state): State<Arc<AuthApiState>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    // Extract refresh token from request
    let refresh_token = data
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Refresh token is required".to_string()))?;

    // Get user ID from token
    let user_id = state.token_service.get_user_id_from_token(refresh_token)?;

    // Call auth service to logout
    state.auth_service.logout(user_id).await?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        "Logged out successfully",
    ))
}

// Verify email handler
pub async fn verify_email(
    Path(token): Path<String>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Verify the token
    let user = state.auth_service.verify_email_token(&token).await?;

    Ok(ApiResponse::success(StatusCode::OK, user))
}

// Request password reset handler
pub async fn request_password_reset(
    State(state): State<Arc<AuthApiState>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    // Extract email from JSON
    let email = data
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Email is required".to_string()))?;

    // Call service to request password reset
    let token = state.auth_service.request_password_reset(email).await?;

    // In a real application, you would send an email with the reset link
    // For simplicity, we'll just acknowledge the request
    Ok(ApiResponse::success(
        StatusCode::OK,
        "Password reset link sent if the email exists in our system",
    ))
}

// Reset password handler
pub async fn reset_password(
    State(state): State<Arc<AuthApiState>>,
    Json(dto): Json<PasswordResetDto>,
) -> Result<Response, AppError> {
    // Validate DTO
    dto.validate().map_err(validation_err_to_app_error)?;

    // Reset the password
    state
        .auth_service
        .reset_password(&dto.token, &dto.new_password)
        .await?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        "Password reset successfully",
    ))
}

// Get current user handler
pub async fn get_current_user(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Get user from claims
    let user_id = claims.sub.parse().unwrap();
    let user = state
        .user_management_service
        .get_user_by_id(user_id)
        .await?;

    Ok(ApiResponse::success(StatusCode::OK, user))
}

// Handler to start the OAuth login process
pub async fn oauth_start(
    Path(provider): Path<String>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Get the authorization URL
    let redirect_url = state.auth_service.get_oauth_redirect_url(&provider).await?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        serde_json::json!({ "url": redirect_url }),
    ))
}

// Handler for OAuth callback
pub async fn oauth_callback(
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Handle error from OAuth provider if present
    if let Some(error) = &query.error {
        return Err(AppError::Authentication(format!("OAuth error: {}", error)));
    }

    // Exchange code for token
    let auth_response = state
        .auth_service
        .handle_oauth_callback(&provider, &query.code)
        .await?;

    Ok(ApiResponse::success(StatusCode::OK, auth_response))
}
