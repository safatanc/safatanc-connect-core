mod auth;
mod health;
mod users;

use std::sync::Arc;

use axum::{
    handler::Handler,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::config::AppConfig;
use crate::db::repositories::Repositories;
use crate::errors::ErrorResponse;
use crate::services::auth::{AuthService, TokenService};
use crate::services::user::UserManagementService;

// Handler for unmatched routes (404 Not Found)
async fn handle_404() -> impl IntoResponse {
    let status = StatusCode::NOT_FOUND;
    let response = ErrorResponse {
        status: status.to_string(),
        message: "Resource not found".to_string(),
    };

    (status, Json(response))
}

// Function to configure all API routes
pub fn configure_api(
    state: Arc<Repositories>,
    config: AppConfig,
    token_service: Arc<TokenService>,
    user_management_service: Arc<UserManagementService>,
    auth_service: Arc<AuthService>,
) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    // Create main router and attach all sub-routers
    Router::new()
        .nest(
            "/users",
            users::configure_users(
                state.clone(),
                config.clone(),
                user_management_service.clone(),
                token_service.clone(),
                auth_service.clone(),
            ),
        )
        // Add auth routes
        .nest(
            "/auth",
            auth::configure(
                state.clone(),
                token_service.clone(),
                user_management_service.clone(),
                auth_service.clone(),
            ),
        )
        // Add additional routes as they are implemented
        // .nest("/health", health::configure_health(...))
        // Add fallback route for handling 404 errors
        .fallback(handle_404)
        // Apply CORS middleware
        .layer(cors)
        // Add middleware for handling method not allowed
        .layer(axum::middleware::map_response(
            |res: axum::response::Response| async move {
                if res.status() == StatusCode::METHOD_NOT_ALLOWED {
                    return (
                        StatusCode::METHOD_NOT_ALLOWED,
                        Json(ErrorResponse {
                            status: StatusCode::METHOD_NOT_ALLOWED.to_string(),
                            message: "Method not allowed for this endpoint".to_string(),
                        }),
                    )
                        .into_response();
                }
                res
            },
        ))
}
