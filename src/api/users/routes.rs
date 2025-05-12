use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::config::AppConfig;
use crate::db::repositories::Repositories;
use crate::middleware::auth::{require_admin, require_auth};
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
    // Create nested router for /users routes with admin-only routes
    let admin_routes = Router::new()
        .route("/", get(handlers::list_users))
        .route("/", post(handlers::create_user))
        .route("/:id", delete(handlers::delete_user))
        .route_layer(middleware::from_fn(require_admin));

    // Create nested router for user routes (accessible to all authenticated users)
    let user_routes = Router::new()
        .route("/me", get(handlers::get_current_user))
        .route("/me", put(handlers::update_current_user))
        .route("/me/password", put(handlers::update_current_user_password))
        .route("/:id", get(handlers::get_user))
        .route("/:id", put(handlers::update_user))
        .route("/:id/password", put(handlers::update_user_password));

    // Merge routes and apply authentication middleware to all
    admin_routes
        .merge(user_routes)
        .route_layer(middleware::from_fn_with_state(
            (state.clone(), token_service.clone()),
            require_auth,
        ))
        .with_state((state, config, user_management_service, auth_service))
}
