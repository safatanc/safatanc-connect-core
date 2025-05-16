use crate::config::{DatabaseConfig, EmailConfig, OAuthConfig};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub oauth: OAuthConfig,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,           // in seconds
    pub refresh_token_expiration: i64, // in seconds
    pub cors_allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Parse CORS origins from comma-separated list
        let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        Self {
            database: DatabaseConfig::from_env(),
            email: EmailConfig::from_env(),
            oauth: OAuthConfig::from_env(),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string()) // 1 hour
                .parse()
                .expect("JWT_EXPIRATION must be a number"),
            refresh_token_expiration: env::var("REFRESH_TOKEN_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .expect("REFRESH_TOKEN_EXPIRATION must be a number"),
            cors_allowed_origins: cors_origins,
        }
    }
}
