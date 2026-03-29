use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(message: impl Into<String>, data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                message: message.into(),
                data: Some(data),
            }),
        )
    }

    pub fn created(message: impl Into<String>, data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::CREATED,
            Json(Self {
                success: true,
                message: message.into(),
                data: Some(data),
            }),
        )
    }
}

impl ApiResponse<()> {
    pub fn message(message: impl Into<String>) -> impl IntoResponse {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                message: message.into(),
                data: None,
            }),
        )
    }
}
