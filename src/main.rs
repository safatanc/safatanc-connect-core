mod api;
mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod services;
mod utils;

use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use db::repositories::OAuthRepository;
use db::repositories::Repositories;
use db::repositories::TokenRepository;
use db::repositories::UserRepository;
use services::auth::{AuthService, OAuthService, TokenService};
use services::badge::BadgeService;
use services::scheduler::SchedulerService;
use services::user::UserManagementService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set up global logger");

    // Load configuration
    let config = config::load_config();
    info!("Configuration loaded");

    // Initialize database connection pool
    let db_pool = db::pool::init_db_pool(&config.database).await?;
    info!("Database connection pool initialized");

    // Run database health check
    db::pool::check_connection(&db_pool).await?;
    info!("Database connection verified");

    // Initialize repositories
    let repos = Arc::new(Repositories::new(db_pool.as_ref().clone()));
    info!("Repositories initialized");

    // Initialize services
    let token_service = Arc::new(TokenService::new(config.clone()));
    let user_repo = UserRepository::new(db_pool.as_ref().clone());
    let token_repo = TokenRepository::new(db_pool.as_ref().clone());
    let oauth_repo = OAuthRepository::new(db_pool.as_ref().clone());

    let user_management_service = Arc::new(UserManagementService::new(user_repo.clone()));

    // Initialize OAuth service
    let oauth_service = Arc::new(OAuthService::new(
        user_repo.clone(),
        oauth_repo,
        token_service.clone(),
        user_management_service.clone(),
    ));

    // Initialize Auth service with OAuth
    let auth_service = Arc::new(
        AuthService::new(
            user_repo,
            token_repo,
            token_service.clone(),
            user_management_service.clone(),
        )
        .with_oauth_service(oauth_service),
    );

    let badge_service = Arc::new(BadgeService::new(repos.clone()));
    info!("Services initialized");

    // Initialize and start scheduler service
    let scheduler = SchedulerService::new(repos.clone());
    scheduler.start_background_tasks();
    info!("Background tasks started");

    // Initialize API router
    let app = api::configure_api(
        repos.clone(),
        config.clone(),
        token_service.clone(),
        user_management_service.clone(),
        auth_service.clone(),
        badge_service.clone(),
    );
    info!("API routes configured");

    // Configure server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Starting server on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
