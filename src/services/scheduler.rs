use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::db::repositories::Repositories;

pub struct SchedulerService {
    repos: Arc<Repositories>,
}

impl SchedulerService {
    pub fn new(repos: Arc<Repositories>) -> Self {
        Self { repos }
    }

    // Start background tasks
    pub fn start_background_tasks(&self) {
        let repos_clone = self.repos.clone();
        tokio::spawn(async move {
            Self::run_token_cleanup(repos_clone).await;
        });
    }

    // Periodically clean up expired tokens
    async fn run_token_cleanup(repos: Arc<Repositories>) {
        let mut interval = time::interval(Duration::from_secs(3600)); // Run every hour
        loop {
            interval.tick().await;
            match repos.token().delete_expired().await {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        tracing::info!("Cleaned up {} expired tokens", result.rows_affected());
                    }
                }
                Err(err) => {
                    tracing::error!("Error cleaning up expired tokens: {:?}", err);
                }
            }
        }
    }
}
