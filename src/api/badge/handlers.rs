use std::sync::Arc;

use crate::db::repositories::Repositories;
use crate::errors::AppError;
use crate::models::badge::{CreateBadgeDto, UpdateBadgeDto};
use crate::models::common::response::ApiResponse;
use crate::models::common::PaginationQuery;
use crate::models::user::AwardBadgeDto;
use crate::services::badge::BadgeService;
use crate::services::validation::validation_err_to_app_error;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use uuid::Uuid;
use validator::Validate;

// Handler to get all badges with pagination
pub async fn get_badges(
    Query(query): Query<PaginationQuery>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    let page = query.page.max(1);
    let limit = query.limit.max(1).min(100);

    let badges = badge_service.get_badges(page, limit).await?;
    Ok(ApiResponse::success(StatusCode::OK, badges))
}

// Handler to get a single badge by ID
pub async fn get_badge(
    Path(id): Path<Uuid>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    let badge = badge_service.get_badge(id).await?;
    Ok(ApiResponse::success(StatusCode::OK, badge))
}

// Handler to create a new badge (admin only)
pub async fn create_badge(
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
    Json(dto): Json<CreateBadgeDto>,
) -> Result<Response, AppError> {
    // Validate DTO
    dto.validate().map_err(validation_err_to_app_error)?;

    let badge = badge_service.create_badge(dto).await?;
    Ok(ApiResponse::created(badge))
}

// Handler to update a badge (admin only)
pub async fn update_badge(
    Path(id): Path<Uuid>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
    Json(dto): Json<UpdateBadgeDto>,
) -> Result<Response, AppError> {
    // Validate DTO
    dto.validate().map_err(validation_err_to_app_error)?;

    let badge = badge_service.update_badge(id, dto).await?;
    Ok(ApiResponse::success(StatusCode::OK, badge))
}

// Handler to delete a badge (admin only)
pub async fn delete_badge(
    Path(id): Path<Uuid>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    badge_service.delete_badge(id).await?;
    Ok(ApiResponse::no_content())
}

// Handler to award a badge to a user (admin only)
pub async fn award_badge(
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
    Json(dto): Json<AwardBadgeDto>,
) -> Result<Response, AppError> {
    // Validate DTO
    dto.validate().map_err(validation_err_to_app_error)?;

    badge_service.award_badge(dto).await?;
    Ok(ApiResponse::created("Badge awarded successfully"))
}

// Handler to remove a badge from a user (admin only)
pub async fn remove_badge(
    Path((user_id, badge_id)): Path<(Uuid, Uuid)>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    badge_service.remove_badge(user_id, badge_id).await?;
    Ok(ApiResponse::no_content())
}

// Handler to get all badges for a user
pub async fn get_user_badges(
    Path(user_id): Path<Uuid>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    let user_badges = badge_service.get_user_badges(user_id).await?;
    Ok(ApiResponse::success(StatusCode::OK, user_badges))
}

// Handler to get all users who have a specific badge
pub async fn get_badge_users(
    Path(badge_id): Path<Uuid>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    let badge_users = badge_service.get_badge_users(badge_id).await?;
    Ok(ApiResponse::success(StatusCode::OK, badge_users))
}

// Handler to check if a user has a specific badge
pub async fn check_user_badge(
    Path((user_id, badge_id)): Path<(Uuid, Uuid)>,
    State((_, badge_service)): State<(Arc<Repositories>, Arc<BadgeService>)>,
) -> Result<Response, AppError> {
    let has_badge = badge_service.check_user_badge(user_id, badge_id).await?;
    Ok(ApiResponse::success(StatusCode::OK, has_badge))
}
