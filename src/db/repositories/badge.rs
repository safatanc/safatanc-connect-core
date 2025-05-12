use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

use crate::db::error::{DatabaseError, DatabaseResult};
use crate::models::badge::{Badge, CreateBadgeDto, UpdateBadgeDto};

#[derive(Clone)]
pub struct BadgeRepository {
    pool: PgPool,
}

impl BadgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Create a new badge
    pub async fn create(&self, dto: &CreateBadgeDto) -> DatabaseResult<Badge> {
        let badge = sqlx::query_as!(
            Badge,
            r#"
            INSERT INTO badges (
                name, description, image_url
            )
            VALUES ($1, $2, $3)
            RETURNING 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            "#,
            dto.name,
            dto.description,
            dto.image_url
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "badges_name_key" => {
                            DatabaseError::Duplicate("Badge name already exists".to_string())
                        }
                        _ => DatabaseError::ConnectionError(e),
                    }
                } else {
                    DatabaseError::ConnectionError(e)
                }
            } else {
                DatabaseError::ConnectionError(e)
            }
        })?;

        Ok(badge)
    }

    // Find badge by ID
    pub async fn find_by_id(&self, id: Uuid) -> DatabaseResult<Badge> {
        let badge = sqlx::query_as!(
            Badge,
            r#"
            SELECT 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            FROM badges
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        badge.ok_or(DatabaseError::NotFound)
    }

    // Find badge by name
    pub async fn find_by_name(&self, name: &str) -> DatabaseResult<Badge> {
        let badge = sqlx::query_as!(
            Badge,
            r#"
            SELECT 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            FROM badges
            WHERE name = $1 AND deleted_at IS NULL
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        badge.ok_or(DatabaseError::NotFound)
    }

    // Get all badges with pagination
    pub async fn find_all(&self, limit: i64, offset: i64) -> DatabaseResult<Vec<Badge>> {
        let badges = sqlx::query_as!(
            Badge,
            r#"
            SELECT 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            FROM badges
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(badges)
    }

    // Count all badges
    pub async fn count(&self) -> DatabaseResult<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM badges
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(count.count.unwrap_or(0))
    }

    // Update badge
    pub async fn update(&self, id: Uuid, dto: &UpdateBadgeDto) -> DatabaseResult<Badge> {
        sqlx::query_as!(
            Badge,
            r#"
            UPDATE badges
            SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                image_url = COALESCE($3, image_url),
                updated_at = now()
            WHERE id = $4 AND deleted_at IS NULL
            RETURNING 
                id, name, description, image_url,
                created_at, updated_at, deleted_at
            "#,
            dto.name,
            dto.description,
            dto.image_url,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if let Some(constraint) = db_err.constraint() {
                    match constraint {
                        "badges_name_key" => {
                            DatabaseError::Duplicate("Badge name already exists".to_string())
                        }
                        _ => DatabaseError::ConnectionError(e),
                    }
                } else {
                    DatabaseError::ConnectionError(e)
                }
            } else {
                DatabaseError::ConnectionError(e)
            }
        })?
        .ok_or(DatabaseError::NotFound)
    }

    // Soft delete a badge
    pub async fn delete(&self, id: Uuid) -> DatabaseResult<PgQueryResult> {
        let result = sqlx::query!(
            r#"
            UPDATE badges
            SET 
                deleted_at = now(),
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
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
