use std::sync::Arc;
use uuid::Uuid;

use crate::db::repositories::Repositories;
use crate::errors::AppError;
use crate::models::badge::{Badge, BadgeResponse, CreateBadgeDto, UpdateBadgeDto};
use crate::models::common::response::PaginatedResponse;
use crate::models::user::{AwardBadgeDto, BadgeWithUsersResponse, UserWithBadgesResponse};
use crate::services::validation::validation_err_to_app_error;
use validator::Validate;

pub struct BadgeService {
    repos: Arc<Repositories>,
}

impl BadgeService {
    pub fn new(repos: Arc<Repositories>) -> Self {
        Self { repos }
    }

    // Create a new badge
    pub async fn create_badge(&self, dto: CreateBadgeDto) -> Result<BadgeResponse, AppError> {
        // Validate the DTO
        dto.validate().map_err(validation_err_to_app_error)?;

        // Create badge in database
        let badge = self.repos.badge().create(&dto).await?;

        Ok(BadgeResponse::from(badge))
    }

    // Get a badge by ID
    pub async fn get_badge(&self, id: Uuid) -> Result<BadgeResponse, AppError> {
        let badge = self.repos.badge().find_by_id(id).await?;
        Ok(BadgeResponse::from(badge))
    }

    // Get a badge by name
    pub async fn get_badge_by_name(&self, name: &str) -> Result<BadgeResponse, AppError> {
        let badge = self.repos.badge().find_by_name(name).await?;
        Ok(BadgeResponse::from(badge))
    }

    // Get all badges with pagination
    pub async fn get_badges(
        &self,
        page: i64,
        limit: i64,
    ) -> Result<PaginatedResponse<BadgeResponse>, AppError> {
        let offset = (page - 1) * limit;
        let badges = self.repos.badge().find_all(limit, offset).await?;
        let total = self.repos.badge().count().await?;

        let badge_responses: Vec<BadgeResponse> = badges.into_iter().map(Badge::into).collect();

        Ok(PaginatedResponse {
            data: badge_responses,
            total,
            page,
            limit,
            total_pages: (total as f64 / limit as f64).ceil() as i64,
        })
    }

    // Update badge
    pub async fn update_badge(
        &self,
        id: Uuid,
        dto: UpdateBadgeDto,
    ) -> Result<BadgeResponse, AppError> {
        // Validate the DTO
        dto.validate().map_err(validation_err_to_app_error)?;

        // Update badge in database
        let badge = self.repos.badge().update(id, &dto).await?;

        Ok(BadgeResponse::from(badge))
    }

    // Delete badge
    pub async fn delete_badge(&self, id: Uuid) -> Result<(), AppError> {
        self.repos.badge().delete(id).await?;
        Ok(())
    }

    // Award a badge to a user
    pub async fn award_badge(&self, dto: AwardBadgeDto) -> Result<(), AppError> {
        // Validate the DTO
        dto.validate().map_err(validation_err_to_app_error)?;

        // Check if user exists
        self.repos.user().find_by_id(dto.user_id).await?;

        // Check if badge exists
        self.repos.badge().find_by_id(dto.badge_id).await?;

        // Check if user already has this badge
        let has_badge = self
            .repos
            .user_badge()
            .has_badge(dto.user_id, dto.badge_id)
            .await?;

        if has_badge {
            return Err(AppError::Validation(
                "User already has this badge".to_string(),
            ));
        }

        // Award badge to user
        self.repos.user_badge().award_badge(&dto).await?;

        Ok(())
    }

    // Remove a badge from a user
    pub async fn remove_badge(&self, user_id: Uuid, badge_id: Uuid) -> Result<(), AppError> {
        // Remove badge from user
        self.repos
            .user_badge()
            .remove_badge(user_id, badge_id)
            .await?;

        Ok(())
    }

    // Get all badges for a user
    pub async fn get_user_badges(&self, user_id: Uuid) -> Result<UserWithBadgesResponse, AppError> {
        // Check if user exists
        self.repos.user().find_by_id(user_id).await?;

        // Get user with all badges
        let user_with_badges = self
            .repos
            .user_badge()
            .get_user_with_badges(user_id)
            .await?;

        Ok(user_with_badges)
    }

    // Get all users who have a specific badge
    pub async fn get_badge_users(
        &self,
        badge_id: Uuid,
    ) -> Result<BadgeWithUsersResponse, AppError> {
        // Check if badge exists
        self.repos.badge().find_by_id(badge_id).await?;

        // Get badge with all users
        let badge_with_users = self
            .repos
            .user_badge()
            .get_badge_with_users(badge_id)
            .await?;

        Ok(badge_with_users)
    }

    // Check if user has a badge
    pub async fn check_user_badge(&self, user_id: Uuid, badge_id: Uuid) -> Result<bool, AppError> {
        // Check if user exists
        self.repos.user().find_by_id(user_id).await?;

        // Check if badge exists
        self.repos.badge().find_by_id(badge_id).await?;

        // Check if user has badge
        let has_badge = self.repos.user_badge().has_badge(user_id, badge_id).await?;

        Ok(has_badge)
    }
}
