use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::repositories::Repositories;
use crate::middleware::auth::require_auth;
use crate::models::user::{CreateUserDto, UpdateUserDto};
use crate::services::auth::{AuthService, TokenService};
use crate::services::user::UserManagementService;

use super::handlers;

pub fn configure(
    state: Arc<Repositories>,
    config: AppConfig,
    user_management_service: Arc<UserManagementService>,
    token_service: Arc<TokenService>,
    auth_service: Arc<AuthService>,
) -> Router {
    // Create nested router for /users routes
    Router::new()
        .route("/", get(handlers::list_users))
        .route("/", post(handlers::create_user))
        .route("/me", get(handlers::get_current_user))
        .route("/:id", get(handlers::get_user))
        .route("/:id", put(handlers::update_user))
        .route("/:id", delete(handlers::delete_user))
        .route("/:id/password", put(handlers::update_password))
        .route_layer(middleware::from_fn_with_state(
            (state.clone(), token_service.clone()),
            require_auth,
        ))
        .with_state((state, config, user_management_service, auth_service))
}
