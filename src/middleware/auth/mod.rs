use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::repositories::Repositories;
use crate::errors::AppError;
use crate::models::user::GLOBAL_ROLE_ADMIN;
use crate::services::auth::TokenService;

// Claims re-export from token service
pub use crate::services::auth::token::Claims;

// Authentication middleware to protect routes
pub async fn require_auth(
    State((repos, token_service)): State<(Arc<Repositories>, Arc<TokenService>)>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the token from the Authorization header
    let token = extract_token_from_headers(&request)
        .ok_or_else(|| AppError::Authentication("Token not found".into()))?;

    // Validate the token and extract claims
    let claims = token_service.verify_token(&token)?;

    // Check if user still exists and is active
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Token contains invalid user ID".into()))?;

    let user = repos
        .user()
        .find_by_id(user_id)
        .await
        .map_err(|_| AppError::Authentication("User not found or inactive".into()))?;

    if !user.is_active {
        return Err(AppError::Authentication("Account is not active".into()));
    }

    // Attach claims to request extensions
    request.extensions_mut().insert(claims);

    // Continue to the handler
    Ok(next.run(request).await)
}

// Admin role check middleware - requires require_auth middleware to run first
pub async fn require_admin(request: Request, next: Next) -> Result<Response, AppError> {
    // Get the claims from extensions (set by require_auth middleware)
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authorization("Authentication required".into()))?;

    // Check if user has admin role
    if claims.role != GLOBAL_ROLE_ADMIN {
        return Err(AppError::Authorization("Admin access required".into()));
    }

    // Continue to the handler
    Ok(next.run(request).await)
}

// Helper function to extract Bearer token from headers
fn extract_token_from_headers(request: &Request) -> Option<String> {
    let auth_header = request.headers().get(header::AUTHORIZATION)?;
    let auth_header = auth_header.to_str().ok()?;

    // Check if it's a Bearer token
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}
