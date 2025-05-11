pub mod oauth;
pub mod session;
pub mod token;
pub mod user;

use std::sync::Arc;
use sqlx::PgPool;

pub use oauth::*;
pub use session::*;
pub use token::*;
pub use user::*;

#[derive(Clone)]
pub struct Repositories {
    user: UserRepository,
    session: SessionRepository,
    oauth: OAuthRepository,
    token: TokenRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user: UserRepository::new(pool.clone()),
            session: SessionRepository::new(pool.clone()),
            oauth: OAuthRepository::new(pool.clone()),
            token: TokenRepository::new(pool),
        }
    }

    pub fn user(&self) -> &UserRepository {
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
