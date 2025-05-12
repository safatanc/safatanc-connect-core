use std::sync::Arc;

use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::repositories::Repositories;
use crate::errors::AppResult;
use crate::middleware::auth::Claims;
use crate::models::response::success_response;
use crate::models::user::{CreateUserDto, UpdateUserDto, UserResponse};
use crate::services::auth::AuthService;
use crate::services::user::UserManagementService;

// Request and response types
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub pages: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// Get all users with pagination
pub async fn list_users(
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationQuery>,
    State((repos, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
) -> AppResult<impl IntoResponse> {
    // Only admin can list all users
    if claims.role != "admin" {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. Only administrators can view the user list.".into(),
        ));
    }

    let (users, total) = user_management
        .get_all_users(pagination.page, pagination.limit)
        .await?;

    let pages = (total as f64 / pagination.limit as f64).ceil() as u64;

    let response = PaginatedResponse {
        data: users,
        total,
        page: pagination.page,
        limit: pagination.limit,
        pages,
    };

    Ok(success_response(StatusCode::OK, response))
}

// Get current user
pub async fn get_current_user(
    Extension(claims): Extension<Claims>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
) -> AppResult<impl IntoResponse> {
    let user_id = Uuid::parse_str(&claims.sub).unwrap();
    let user = user_management.get_user_by_id(user_id).await?;
    Ok(success_response(StatusCode::OK, user))
}

// Get user by ID
pub async fn get_user(
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
) -> AppResult<impl IntoResponse> {
    // Users can only access their own data, unless they are admin
    if claims.sub != id.to_string() && claims.role != "admin" {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. You can only view your own data.".into(),
        ));
    }

    let user = user_management.get_user_by_id(id).await?;
    Ok(success_response(StatusCode::OK, user))
}

// Create a new user (admin only)
pub async fn create_user(
    Extension(claims): Extension<Claims>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
    Json(create_dto): Json<CreateUserDto>,
) -> AppResult<impl IntoResponse> {
    // Only admin can create users directly
    if claims.role != "admin" {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. Only administrators can create new users.".into(),
        ));
    }

    let user = user_management.register_user(create_dto).await?;
    let user_response = UserResponse::from(user);

    Ok(success_response(StatusCode::CREATED, user_response))
}

// Update user
pub async fn update_user(
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
    Json(update_dto): Json<UpdateUserDto>,
) -> AppResult<impl IntoResponse> {
    // Users can only update their own data, unless they are admin
    if claims.sub != id.to_string() && claims.role != "admin" {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. You can only modify your own data.".into(),
        ));
    }

    // If not admin, they can't modify the is_active field
    if claims.role != "admin" && update_dto.is_active.is_some() {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. Only administrators can change a user's active status.".into(),
        ));
    }

    let user = user_management.update_user(id, update_dto).await?;
    Ok(success_response(StatusCode::OK, user))
}

// Delete user (soft delete)
pub async fn delete_user(
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
) -> AppResult<impl IntoResponse> {
    // Only admin can delete users
    if claims.role != "admin" {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. Only administrators can delete users.".into(),
        ));
    }

    user_management.delete_user(id).await?;
    Ok(success_response(
        StatusCode::OK,
        serde_json::json!({ "message": "User successfully deleted" }),
    ))
}

// Update password
pub async fn update_password(
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    State((_, _, user_management, _)): State<(
        Arc<Repositories>,
        AppConfig,
        Arc<UserManagementService>,
        Arc<AuthService>,
    )>,
    Json(password_request): Json<UpdatePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    // Users can only update their own password
    if claims.sub != id.to_string() {
        return Err(crate::errors::AppError::Authorization(
            "Access denied. You can only change your own password.".into(),
        ));
    }

    user_management
        .update_password(
            id,
            &password_request.current_password,
            &password_request.new_password,
        )
        .await?;

    Ok(success_response(
        StatusCode::OK,
        serde_json::json!({ "message": "Password updated successfully" }),
    ))
}
