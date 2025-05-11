use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            connection_string: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("DB_MAX_CONNECTIONS must be a number"),
        }
    }
}
