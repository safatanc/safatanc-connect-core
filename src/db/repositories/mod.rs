pub mod oauth;
pub mod session;
pub mod token;
pub mod user;

use std::sync::Arc;
use sqlx::PgPool;
use crate::db::executor::DbExecutor;

pub use oauth::*;
pub use session::*;
pub use token::*;
pub use user::*;

#[derive(Clone)]
pub struct Repositories {
    user: Arc<UserRepository<PgPool>>,
    session: SessionRepository,
    oauth: OAuthRepository,
    token: TokenRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        let pool_arc = Arc::new(pool);
        
        Self {
            user: Arc::new(UserRepository::new(pool_arc.clone())),
            session: SessionRepository::new(pool_arc.clone()),
            oauth: OAuthRepository::new(pool_arc.clone()),
            token: TokenRepository::new(pool_arc),
        }
    }

    pub fn user(&self) -> &Arc<UserRepository<PgPool>> {
        &self.user
    }

    pub fn session(&self) -> &SessionRepository {
        &self.session
    }

    pub fn oauth(&self) -> &OAuthRepository {
        &self.oauth
    }
    
    pub fn token(&self) -> &TokenRepository {
        &self.token
    }
}
