use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub status: String,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl ApiResponse {
    pub fn success<T: Serialize>(status_code: StatusCode, data: T) -> Response {
        let response = Self {
            success: true,
            message: None,
            data: Some(serde_json::to_value(data).unwrap()),
        };

        (status_code, Json(response)).into_response()
    }

    pub fn created<T: Serialize>(data: T) -> Response {
        Self::success(StatusCode::CREATED, data)
    }

    pub fn no_content() -> Response {
        let response = Self {
            success: true,
            message: None,
            data: None,
        };

        (StatusCode::NO_CONTENT, Json(response)).into_response()
    }

    pub fn error(status_code: StatusCode, message: String) -> Response {
        let response = Self {
            success: false,
            message: Some(message),
            data: None,
        };

        (status_code, Json(response)).into_response()
    }
}
