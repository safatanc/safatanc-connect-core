use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::db::repositories::Repositories;
use crate::middleware::auth::{require_admin, require_auth};
use crate::services::auth::TokenService;
use crate::services::badge::BadgeService;

use super::handlers;

// Configure badge routes
pub fn configure(
    repo: Arc<Repositories>,
    token_service: Arc<TokenService>,
    badge_service: Arc<BadgeService>,
) -> Router {
    // Public routes - no auth required
    let public_routes = Router::new()
        .route("/", get(handlers::get_badges))
        .route("/:id", get(handlers::get_badge));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/", post(handlers::create_badge))
        .route("/:id", put(handlers::update_badge))
        .route("/:id", delete(handlers::delete_badge))
        .route("/award", post(handlers::award_badge))
        .route(
            "/users/:user_id/badges/:badge_id",
            delete(handlers::remove_badge),
        )
        .route_layer(middleware::from_fn(require_admin));

    // User routes - auth required
    let user_routes = Router::new()
        .route("/:id/users", get(handlers::get_badge_users))
        .route("/users/:user_id", get(handlers::get_user_badges))
        .route(
            "/users/:user_id/badges/:badge_id/check",
            get(handlers::check_user_badge),
        );

    // Combine auth routes and apply auth middleware
    let auth_routes = admin_routes
        .merge(user_routes)
        .route_layer(middleware::from_fn_with_state(
            (repo.clone(), token_service.clone()),
            require_auth,
        ));

    // Merge all routes
    public_routes
        .merge(auth_routes)
        .with_state((repo, badge_service))
}
