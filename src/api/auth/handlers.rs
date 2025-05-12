use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use axum_extra::extract::TypedHeader;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use validator::Validate;

use super::routes::AuthApiState;
use crate::errors::AppError;
use crate::models::response::{error_response, success_response};
use crate::models::user::{
    CreateUserDto, LoginDto, PasswordResetDto, PasswordResetRequestDto, UserResponse,
};
use crate::services::validation::validation_err_to_app_error;

// Login handler
pub async fn login(
    State(state): State<Arc<AuthApiState>>,
    data: Result<Json<LoginDto>, JsonRejection>,
) -> impl IntoResponse {
    // Handle possible JSON extraction errors
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, format!("Invalid request: {}", err));
        }
    };

    // Client-side validation
    if let Err(err) = data.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            validation_err_to_app_error(err).to_string(),
        );
    }

    // Handle login
    match state.auth_service.login(&data.0).await {
        Ok(response) => success_response(StatusCode::OK, response),
        Err(err) => handle_error(&err),
    }
}

// Registration handler
pub async fn register(
    State(state): State<Arc<AuthApiState>>,
    data: Result<Json<CreateUserDto>, JsonRejection>,
) -> impl IntoResponse {
    // Handle possible JSON extraction errors
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, format!("Invalid request: {}", err));
        }
    };

    // Client-side validation
    if let Err(err) = data.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            validation_err_to_app_error(err).to_string(),
        );
    }

    // Handle registration
    match state
        .auth_service
        .register_user_with_verification(data.0)
        .await
    {
        Ok((user, _token)) => {
            // We don't send the token directly in the response
            // Instead, we would typically send an email with the verification link
            // For now, just return the created user
            success_response(StatusCode::CREATED, UserResponse::from(user))
        }
        Err(err) => handle_error(&err),
    }
}

// Token refresh handler
pub async fn refresh_token(
    State(state): State<Arc<AuthApiState>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    // Get the refresh token from the Authorization header
    let refresh_token = bearer.token();

    // Handle token refresh
    match state.auth_service.refresh_access_token(refresh_token).await {
        Ok(new_token) => {
            // Return just the new access token
            success_response(StatusCode::OK, serde_json::json!({ "token": new_token }))
        }
        Err(err) => handle_error(&err),
    }
}

// Logout handler
pub async fn logout(
    State(state): State<Arc<AuthApiState>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    // Verify the token and get the user ID
    let user_id = match state.auth_service.verify_token(bearer.token()) {
        Ok(user_id) => user_id,
        Err(err) => return handle_error(&err),
    };

    // Handle logout
    match state.auth_service.logout(user_id).await {
        Ok(_) => success_response(
            StatusCode::OK,
            serde_json::json!({ "message": "Logged out successfully" }),
        ),
        Err(err) => handle_error(&err),
    }
}

// Email verification handler
pub async fn verify_email(
    State(state): State<Arc<AuthApiState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    // Get the token from the path
    match state.auth_service.verify_email_token(&token).await {
        Ok(user) => success_response(
            StatusCode::OK,
            serde_json::json!({
                "message": "Email verified successfully",
                "user": user
            }),
        ),
        Err(err) => handle_error(&err),
    }
}

// Password reset request handler
pub async fn request_password_reset(
    State(state): State<Arc<AuthApiState>>,
    data: Result<Json<PasswordResetRequestDto>, JsonRejection>,
) -> impl IntoResponse {
    // Handle possible JSON extraction errors
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, format!("Invalid request: {}", err));
        }
    };

    // Client-side validation
    if let Err(err) = data.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            validation_err_to_app_error(err).to_string(),
        );
    }

    // Handle password reset request
    match state.auth_service.request_password_reset(&data.email).await {
        Ok(_) => {
            // Don't leak information about whether email exists
            success_response(
                StatusCode::OK,
                serde_json::json!({
                    "message": "If the email exists, a password reset link has been sent"
                }),
            )
        }
        Err(err) => {
            // For security, even if there's an error, don't reveal it
            // Just log it internally and return a generic message
            eprintln!("Password reset request error: {:?}", err);
            success_response(
                StatusCode::OK,
                serde_json::json!({
                    "message": "If the email exists, a password reset link has been sent"
                }),
            )
        }
    }
}

// Password reset handler
pub async fn reset_password(
    State(state): State<Arc<AuthApiState>>,
    data: Result<Json<PasswordResetDto>, JsonRejection>,
) -> impl IntoResponse {
    // Handle possible JSON extraction errors
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            return error_response(StatusCode::BAD_REQUEST, format!("Invalid request: {}", err));
        }
    };

    // Client-side validation
    if let Err(err) = data.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            validation_err_to_app_error(err).to_string(),
        );
    }

    // Handle password reset
    match state
        .auth_service
        .reset_password(&data.token, &data.new_password)
        .await
    {
        Ok(_) => success_response(
            StatusCode::OK,
            serde_json::json!({
                "message": "Password reset successful"
            }),
        ),
        Err(err) => handle_error(&err),
    }
}

// Get current user handler
pub async fn get_current_user(
    State(state): State<Arc<AuthApiState>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    // Verify the token and get the user ID
    let user_id = match state.auth_service.verify_token(bearer.token()) {
        Ok(user_id) => user_id,
        Err(err) => return handle_error(&err),
    };

    // Get user details
    match state.user_management_service.get_user_by_id(user_id).await {
        Ok(user) => success_response(StatusCode::OK, user),
        Err(err) => handle_error(&err),
    }
}

// OAuth start handler - redirects to the provider's authorization page
pub async fn oauth_start(
    State(state): State<Arc<AuthApiState>>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    match state.auth_service.get_oauth_redirect_url(&provider).await {
        Ok(authorization_url) => success_response(
            StatusCode::OK,
            serde_json::json!({ "authorization_url": authorization_url.to_string() }),
        ),
        Err(err) => handle_error(&err),
    }
}

// OAuth callback handler - processes the OAuth callback
pub async fn oauth_callback(
    State(state): State<Arc<AuthApiState>>,
    Path(provider): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Ensure we have the authorization code from the callback
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "Missing authorization code".to_string(),
            );
        }
    };

    // Exchange the code for tokens and authenticate/register the user
    match state
        .auth_service
        .handle_oauth_callback(&provider, code)
        .await
    {
        Ok(auth_response) => success_response(StatusCode::OK, auth_response),
        Err(err) => handle_error(&err),
    }
}

// Helper function to handle errors
fn handle_error(err: &AppError) -> Response {
    let status = match err {
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
        AppError::Authorization(_) => StatusCode::FORBIDDEN,
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
        AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    error_response(status, err.to_string())
}
