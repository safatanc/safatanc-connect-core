use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::badge::BadgeResponse;
use crate::models::user::UserResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBadge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AwardBadgeDto {
    pub user_id: Uuid,
    pub badge_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UserBadgeResponse {
    pub id: Uuid,
    pub user: UserResponse,
    pub badge: BadgeResponse,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserWithBadgesResponse {
    pub user: UserResponse,
    pub badges: Vec<BadgeResponse>,
}

#[derive(Debug, Serialize)]
pub struct BadgeWithUsersResponse {
    pub badge: BadgeResponse,
    pub users: Vec<UserResponse>,
}
