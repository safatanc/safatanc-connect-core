use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::db::repositories::Repositories;
use crate::middleware::auth::{require_auth, require_verified_email};
use crate::services::auth::{AuthService, TokenService};
use crate::services::email::EmailService;
use crate::services::user::UserManagementService;

use super::handlers;

// Auth API State struct
pub struct AuthApiState {
    pub token_service: Arc<TokenService>,
    pub user_management_service: Arc<UserManagementService>,
    pub auth_service: Arc<AuthService>,
    pub email_service: Arc<EmailService>,
}

// Configure auth routes
pub fn configure(
    repos: Arc<Repositories>,
    token_service: Arc<TokenService>,
    user_management_service: Arc<UserManagementService>,
    auth_service: Arc<AuthService>,
    email_service: Arc<EmailService>,
) -> Router {
    let state = Arc::new(AuthApiState {
        token_service: token_service.clone(),
        user_management_service,
        auth_service,
        email_service,
    });

    // Public routes - no auth required
    let public_routes = Router::new()
        .route("/login", post(handlers::login))
        .route("/register", post(handlers::register))
        .route("/refresh", post(handlers::refresh_token))
        .route("/verify-email/:token", get(handlers::verify_email))
        .route(
            "/request-password-reset",
            post(handlers::request_password_reset),
        )
        .route("/reset-password", post(handlers::reset_password))
        .route("/oauth/:provider", get(handlers::oauth_start))
        .route("/oauth/:provider/callback", get(handlers::oauth_callback));

    // Auth routes that don't require email verification
    let unverified_auth_routes = Router::new()
        .route(
            "/resend-verification-email",
            post(handlers::resend_verification_email),
        )
        .route_layer(middleware::from_fn_with_state(
            (repos.clone(), token_service.clone()),
            require_auth,
        ));

    // Auth routes that require email verification
    let verified_auth_routes = Router::new()
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::get_current_user))
        .route_layer(middleware::from_fn_with_state(
            repos.clone(),
            require_verified_email,
        ))
        .route_layer(middleware::from_fn_with_state(
            (repos, token_service),
            require_auth,
        ));

    // Merge all routes
    public_routes
        .merge(unverified_auth_routes)
        .merge(verified_auth_routes)
        .with_state(state)
}
