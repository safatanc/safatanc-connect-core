mod api;
mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod services;
mod utils;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use db::repositories::Repositories;
use db::repositories::TokenRepository;
use db::repositories::UserRepository;
use services::auth::{AuthService, TokenService};
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

    let user_management_service = Arc::new(UserManagementService::new(user_repo.clone()));
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        token_repo,
        token_service.clone(),
        user_management_service.clone(),
    ));
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

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("Starting server on http://localhost:{}", config.server_port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
