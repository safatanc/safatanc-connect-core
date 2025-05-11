use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::db::repositories::Repositories;
use crate::services::auth::{AuthService, TokenService};
use crate::services::user::UserManagementService;

use super::handlers;

// Auth API State struct
pub struct AuthApiState {
    pub repos: Arc<Repositories>,
    pub token_service: Arc<TokenService>,
    pub user_management_service: Arc<UserManagementService>,
    pub auth_service: Arc<AuthService>,
}

// Configure auth routes
pub fn configure(
    repos: Arc<Repositories>,
    token_service: Arc<TokenService>,
    user_management_service: Arc<UserManagementService>,
    auth_service: Arc<AuthService>,
) -> Router {
    let state = AuthApiState {
        repos,
        token_service,
        user_management_service,
        auth_service,
    };

    Router::new()
        .route("/login", post(handlers::login))
        .route("/register", post(handlers::register))
        .route("/refresh", post(handlers::refresh_token))
        .route("/logout", post(handlers::logout))
        .route("/verify-email/:token", get(handlers::verify_email))
        .route(
            "/request-password-reset",
            post(handlers::request_password_reset),
        )
        .route("/reset-password", post(handlers::reset_password))
        .route("/me", get(handlers::get_current_user))
        .route("/oauth/:provider", get(handlers::oauth_start))
        .route("/oauth/:provider/callback", get(handlers::oauth_callback))
        .with_state(Arc::new(state))
}
