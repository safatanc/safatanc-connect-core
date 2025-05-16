mod app;
mod database;
mod email;
mod oauth;

pub use app::AppConfig;
pub use database::DatabaseConfig;
pub use email::EmailConfig;
pub use oauth::OAuthConfig;

use dotenv::dotenv;

pub fn load_config() -> AppConfig {
    // Load environment variables from .env file if it exists
    dotenv().ok();

    AppConfig::from_env()
}
