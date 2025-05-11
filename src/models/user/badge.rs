use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Badge {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserBadge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBadgeDto {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBadgeDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

// Implementasi From trait untuk konversi dari Badge ke BadgeResponse
impl From<Badge> for BadgeResponse {
    fn from(badge: Badge) -> Self {
        Self {
            id: badge.id,
            name: badge.name,
            description: badge.description,
            image_url: badge.image_url,
        }
    }
}
