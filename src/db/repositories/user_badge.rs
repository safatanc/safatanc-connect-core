use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::badge::{Badge, BadgeResponse};
use crate::models::user::{
    AwardBadgeDto, BadgeWithUsersResponse, User, UserBadge, UserResponse, UserWithBadgesResponse,
};

#[derive(Clone)]
pub struct UserBadgeRepository {
    pool: PgPool,
}

impl UserBadgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Award a badge to a user
    pub async fn award_badge(&self, dto: &AwardBadgeDto) -> DatabaseResult<UserBadge> {
        let user_badge = sqlx::query_as!(
            UserBadge,
            r#"
            INSERT INTO user_badges (
                user_id, badge_id
            )
            VALUES ($1, $2)
            RETURNING 
                id, user_id, badge_id, created_at, updated_at, deleted_at
            "#,
            dto.user_id,
            dto.badge_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "user_badges_user_id_badge_id_key" => {
                            DatabaseError::Duplicate("User already has this badge".to_string())
                        }
                        "user_badges_user_id_fkey" => DatabaseError::NotFound,
                        "user_badges_badge_id_fkey" => DatabaseError::NotFound,
                        _ => DatabaseError::ConnectionError(e),
                    }
                } else {
                    DatabaseError::ConnectionError(e)
                }
            } else {
                DatabaseError::ConnectionError(e)
            }
        })?;

        Ok(user_badge)
    }

    // Find user_badge by ID
    pub async fn find_by_id(&self, id: Uuid) -> DatabaseResult<UserBadge> {
        let user_badge = sqlx::query_as!(
            UserBadge,
            r#"
            SELECT 
                id, user_id, badge_id, created_at, updated_at, deleted_at
            FROM user_badges
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        user_badge.ok_or(DatabaseError::NotFound)
    }

    // Find all badges for a specific user
    pub async fn find_badges_by_user_id(&self, user_id: Uuid) -> DatabaseResult<Vec<Badge>> {
        let badges = sqlx::query_as!(
            Badge,
            r#"
            SELECT 
                b.id, b.name, b.description, b.image_url,
                b.created_at, b.updated_at, b.deleted_at
            FROM badges b
            JOIN user_badges ub ON b.id = ub.badge_id
            WHERE ub.user_id = $1 
              AND ub.deleted_at IS NULL
              AND b.deleted_at IS NULL
            ORDER BY ub.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(badges)
    }

    // Find all users who have a specific badge
    pub async fn find_users_by_badge_id(&self, badge_id: Uuid) -> DatabaseResult<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                u.id, u.email, u.username, u.password_hash, u.full_name, u.avatar_url,
                u.global_role, u.is_email_verified, u.is_active, u.last_login_at,
                u.created_at, u.updated_at, u.deleted_at
            FROM users u
            JOIN user_badges ub ON u.id = ub.user_id
            WHERE ub.badge_id = $1 
              AND ub.deleted_at IS NULL
              AND u.deleted_at IS NULL
            ORDER BY ub.created_at DESC
            "#,
            badge_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(users)
    }

    // Check if a user has a specific badge
    pub async fn has_badge(&self, user_id: Uuid, badge_id: Uuid) -> DatabaseResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM user_badges
                WHERE user_id = $1 AND badge_id = $2 AND deleted_at IS NULL
            ) as exists
            "#,
            user_id,
            badge_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(result.exists.unwrap_or(false))
    }

    // Get user with all their badges
    pub async fn get_user_with_badges(
        &self,
        user_id: Uuid,
    ) -> DatabaseResult<UserWithBadgesResponse> {
        // First get the user
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, username, password_hash, full_name, avatar_url,
                global_role, is_email_verified, is_active, last_login_at,
                created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?
        .ok_or(DatabaseError::NotFound)?;

        // Then get all badges for the user
        let badges = self.find_badges_by_user_id(user_id).await?;

        let badge_responses: Vec<BadgeResponse> =
            badges.into_iter().map(BadgeResponse::from).collect();

        Ok(UserWithBadgesResponse {
            user: UserResponse::from(user),
            badges: badge_responses,
        })
    }

    // Get badge with all users who have it
    pub async fn get_badge_with_users(
        &self,
        badge_id: Uuid,
    ) -> DatabaseResult<BadgeWithUsersResponse> {
        // First get the badge
        let badge = sqlx::query_as!(
            Badge,
            r#"
            SELECT 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            FROM badges
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            badge_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?
        .ok_or(DatabaseError::NotFound)?;

        // Then get all users who have this badge
        let users = self.find_users_by_badge_id(badge_id).await?;

        let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

        Ok(BadgeWithUsersResponse {
            badge: BadgeResponse::from(badge),
            users: user_responses,
        })
    }

    // Remove a badge from a user
    pub async fn remove_badge(
        &self,
        user_id: Uuid,
        badge_id: Uuid,
    ) -> DatabaseResult<PgQueryResult> {
        let result = sqlx::query!(
            r#"
            UPDATE user_badges
            SET 
                deleted_at = now(),
                updated_at = now()
            WHERE user_id = $1 AND badge_id = $2 AND deleted_at IS NULL
            "#,
            user_id,
            badge_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::NotFound);
        }

        Ok(result)
    }
}
