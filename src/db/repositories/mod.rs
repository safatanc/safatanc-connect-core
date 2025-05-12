pub mod badge;
pub mod oauth;
pub mod session;
pub mod token;
pub mod user;
pub mod user_badge;

use sqlx::PgPool;

pub use badge::*;
pub use oauth::*;
pub use session::*;
pub use token::*;
pub use user::*;
pub use user_badge::*;

#[derive(Clone)]
pub struct Repositories {
    user: UserRepository,
    session: SessionRepository,
    oauth: OAuthRepository,
    token: TokenRepository,
    badge: BadgeRepository,
    user_badge: UserBadgeRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user: UserRepository::new(pool.clone()),
            session: SessionRepository::new(pool.clone()),
            oauth: OAuthRepository::new(pool.clone()),
            token: TokenRepository::new(pool.clone()),
            badge: BadgeRepository::new(pool.clone()),
            user_badge: UserBadgeRepository::new(pool),
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

    pub fn badge(&self) -> &BadgeRepository {
        &self.badge
    }

    pub fn user_badge(&self) -> &UserBadgeRepository {
        &self.user_badge
    }
}
