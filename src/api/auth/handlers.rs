use std::sync::Arc;

use axum::extract::Extension;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use validator::Validate;

use super::routes::AuthApiState;
use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::auth::oauth::{OAuthCallbackQuery, OAuthStartQuery};
use crate::models::common::response::ApiResponse;
use crate::models::user::{
    CreateUserDto, LoginDto, PasswordResetDto, ResendVerificationEmailDto, UserResponse,
};
use crate::services::validation::validation_err_to_app_error;

// Login handler
pub async fn login(
    State(state): State<Arc<AuthApiState>>,
    Json(credentials): Json<LoginDto>,
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
    let user = state
        .user_management_service
        .register_user(dto.clone())
        .await?;

    // Send verification email (non-blocking)
    state
        .email_service
        .send_verification_email(user.id, &user.email, &user.username)
        .await?;

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

    // Get user by email
    let user = state
        .user_management_service
        .get_user_by_email(email)
        .await?;

    // Send password reset email (non-blocking)
    state
        .email_service
        .send_password_reset_email(email, &user.username, &token)
        .await?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        "Password reset link sent if the email exists in our system",
    ))
}

// Resend verification email handler
pub async fn resend_verification_email(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Get user from claims
    let user_id = claims.sub.parse().unwrap();
    let user = state
        .user_management_service
        .get_user_by_id(user_id)
        .await?;

    // Check if email is already verified
    if user.is_email_verified {
        return Err(AppError::Validation(
            "Email is already verified".to_string(),
        ));
    }

    // Send verification email (non-blocking)
    state
        .email_service
        .send_verification_email(user.id, &user.email, &user.username)
        .await?;

    Ok(ApiResponse::success(
        StatusCode::OK,
        "Verification email sent",
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
    Query(query): Query<OAuthStartQuery>,
    State(state): State<Arc<AuthApiState>>,
) -> Result<Response, AppError> {
    // Get the authorization URL with a basic state parameter
    let mut auth_url = state.auth_service.get_oauth_redirect_url(&provider).await?;

    // If redirect_uri is provided, modify the state parameter to include it
    if let Some(redirect_uri) = &query.redirect_uri {
        // Check if the URL already has a state parameter
        if auth_url.contains("state=") {
            // Extract the existing state value
            let parts: Vec<&str> = auth_url.split("state=").collect();
            if parts.len() > 1 {
                let state_parts: Vec<&str> = parts[1].split('&').collect();
                let existing_state = state_parts[0];

                // Create a new state that encodes both values
                let custom_redirect = urlencoding::encode(redirect_uri).into_owned();
                let combined_state = if existing_state.is_empty() {
                    custom_redirect
                } else {
                    format!("{}_redirect_{}", existing_state, custom_redirect)
                };

                // Replace the state parameter
                let old_state_param = format!("state={}", existing_state);
                let new_state_param = format!("state={}", combined_state);
                auth_url = auth_url.replacen(&old_state_param, &new_state_param, 1);
            }
        } else {
            // Add state parameter with encoded redirect if no state exists
            let separator = if auth_url.contains('?') { '&' } else { '?' };
            let encoded_redirect = urlencoding::encode(redirect_uri).into_owned();
            auth_url.push_str(&format!("{}state={}", separator, encoded_redirect));
        }
    }

    Ok(ApiResponse::success(
        StatusCode::OK,
        serde_json::json!({ "url": auth_url }),
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

    // Parse the state param to extract custom_redirect if present
    let custom_redirect = if let Some(state_param) = &query.state {
        if state_param.contains("_redirect_") {
            // Extract the redirect part after _redirect_ marker
            let parts: Vec<&str> = state_param.split("_redirect_").collect();
            if parts.len() > 1 {
                urlencoding::decode(parts[1]).map(|s| s.to_string()).ok()
            } else {
                None
            }
        } else {
            // The whole state might be the redirect URI
            urlencoding::decode(state_param).map(|s| s.to_string()).ok()
        }
    } else {
        None
    };

    // Exchange code for token
    let auth_response = state
        .auth_service
        .handle_oauth_callback(&provider, &query.code)
        .await?;

    let frontend_url = state.config.email.frontend_url.clone();

    // Determine the redirect URL
    // Always redirect to frontend callback first, passing redirect_uri as a query parameter
    let redirect_url = if let Some(redirect_uri) = query.redirect_uri {
        format!(
            "{}/auth/callback?redirect_uri={}&token={}&refresh_token={}",
            frontend_url.trim_end_matches('/'),
            urlencoding::encode(&redirect_uri),
            auth_response.token,
            auth_response.refresh_token
        )
    } else if let Some(custom_redirect) = custom_redirect {
        format!(
            "{}/auth/callback?redirect_uri={}&token={}&refresh_token={}",
            frontend_url.trim_end_matches('/'),
            urlencoding::encode(&custom_redirect),
            auth_response.token,
            auth_response.refresh_token
        )
    } else {
        // Default to frontend URL with /auth/callback without redirect_uri
        format!(
            "{}/auth/callback?token={}&refresh_token={}",
            frontend_url.trim_end_matches('/'),
            auth_response.token,
            auth_response.refresh_token
        )
    };

    // Redirect to frontend with tokens
    Ok(Redirect::to(&redirect_url).into_response())
}
