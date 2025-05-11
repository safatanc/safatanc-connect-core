pub mod user;

use sqlx::PgPool;
pub use user::*;

#[derive(Clone)]
pub struct Repositories {
    user: UserRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user: UserRepository::new(pool),
        }
    }

    pub fn user(&self) -> &UserRepository {
        &self.user
    }
}
