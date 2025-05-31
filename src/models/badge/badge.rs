use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBadgeDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Badge name must be between 1 and 100 characters"
    ))]
    pub name: String,

    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBadgeDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Badge name must be between 1 and 100 characters"
    ))]
    pub name: Option<String>,

    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BadgeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Implementation of From trait for converting from Badge to BadgeResponse
impl From<Badge> for BadgeResponse {
    fn from(badge: Badge) -> Self {
        Self {
            id: badge.id,
            name: badge.name,
            description: badge.description,
            image_url: badge.image_url,
            created_at: badge.created_at,
        }
    }
}
