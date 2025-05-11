use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::TypedHeader;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;

use super::routes::AuthApiState;
use crate::errors::{AppError, ErrorResponse};
use crate::models::auth::token::VerificationToken;
use crate::models::user::{
    AuthResponse, CreateUserDto, LoginDto, PasswordResetDto, PasswordResetRequestDto, UserResponse,
};

// Login handler
pub async fn login(
    State(state): State<Arc<AuthApiState>>,
    data: Result<Json<LoginDto>, JsonRejection>,
) -> impl IntoResponse {
    // Handle possible JSON extraction errors
    let data = match data {
        Ok(data) => data,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("Invalid request: {}", err),
                }),
            )
                .into_response()
        }
    };

    // Handle login
    match state.auth_service.login(&data.0).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => handle_error(&err).into_response(),
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
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("Invalid request: {}", err),
                }),
            )
                .into_response()
        }
    };

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
            (StatusCode::CREATED, Json(UserResponse::from(user))).into_response()
        }
        Err(err) => handle_error(&err).into_response(),
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
            (
                StatusCode::OK,
                Json(serde_json::json!({ "token": new_token })),
            )
                .into_response()
        }
        Err(err) => handle_error(&err).into_response(),
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
        Err(err) => return handle_error(&err).into_response(),
    };

    // Handle logout
    match state.auth_service.logout(user_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => handle_error(&err).into_response(),
    }
}

// Email verification handler
pub async fn verify_email(
    State(state): State<Arc<AuthApiState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    // Get the token from the path
    match state.auth_service.verify_email_token(&token).await {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": "Email verified successfully",
                "user": user
            })),
        )
            .into_response(),
        Err(err) => handle_error(&err).into_response(),
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
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("Invalid request: {}", err),
                }),
            )
                .into_response()
        }
    };

    // Handle password reset request
    match state.auth_service.request_password_reset(&data.email).await {
        Ok(_) => {
            // Don't leak information about whether email exists
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "message": "If the email exists, a password reset link has been sent"
                })),
            )
                .into_response()
        }
        Err(err) => {
            // For security, even if there's an error, don't reveal it
            // Just log it internally and return a generic message
            eprintln!("Password reset request error: {:?}", err);
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "success",
                    "message": "If the email exists, a password reset link has been sent"
                })),
            )
                .into_response()
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
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("Invalid request: {}", err),
                }),
            )
                .into_response()
        }
    };

    // Handle password reset
    match state
        .auth_service
        .reset_password(&data.token, &data.new_password)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": "Password reset successful"
            })),
        )
            .into_response(),
        Err(err) => handle_error(&err).into_response(),
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
        Err(err) => return handle_error(&err).into_response(),
    };

    // Get user details
    match state.user_management_service.get_user_by_id(user_id).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(err) => handle_error(&err).into_response(),
    }
}

// OAuth start handler - redirects to the provider's authorization page
pub async fn oauth_start(
    State(state): State<Arc<AuthApiState>>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    match state.auth_service.get_oauth_redirect_url(&provider).await {
        Ok(url) => axum::response::Redirect::to(&url).into_response(),
        Err(err) => handle_error(&err).into_response(),
    }
}

// OAuth callback handler - handles the callback from the provider
pub async fn oauth_callback(
    State(state): State<Arc<AuthApiState>>,
    Path(provider): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Ensure we have the authorization code from the callback
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: StatusCode::BAD_REQUEST.to_string(),
                    message: "Missing authorization code".to_string(),
                }),
            )
                .into_response()
        }
    };

    // Exchange the code for tokens and authenticate/register the user
    match state
        .auth_service
        .handle_oauth_callback(&provider, code)
        .await
    {
        Ok(auth_response) => (StatusCode::OK, Json(auth_response)).into_response(),
        Err(err) => handle_error(&err).into_response(),
    }
}

// Helper function to handle errors
fn handle_error(err: &AppError) -> (StatusCode, Json<ErrorResponse>) {
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

    (
        status,
        Json(ErrorResponse {
            status: status.to_string(),
            message: err.to_string(),
        }),
    )
}
