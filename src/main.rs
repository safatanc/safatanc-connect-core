mod api;
mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod services;
mod utils;

use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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
    let db_pool = db::init_db_pool(&config.database).await?;
    info!("Database connection pool initialized");

    // Run database health check
    db::check_connection(&db_pool).await?;
    info!("Database connection verified");

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("Starting server on {}", addr);

    // TODO: Initialize and start the server with axum
    println!("Safatanc Connect API - Auth, SSO, and User Management");
    println!("Server will be implemented in the next step");

    Ok(())
}
