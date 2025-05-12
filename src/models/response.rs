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

pub fn success_response<T: Serialize>(status_code: StatusCode, data: T) -> Response {
    let response = SuccessResponse {
        status: status_code.to_string(),
        data,
    };

    (status_code, Json(response)).into_response()
}

pub fn error_response(status_code: StatusCode, message: String) -> Response {
    let response = ErrorResponse {
        status: status_code.to_string(),
        message,
    };

    (status_code, Json(response)).into_response()
}
