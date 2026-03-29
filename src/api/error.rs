use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;
use tracing::error;
use validator::ValidationErrors;

use crate::domain::errors::DomainError;

#[derive(Debug, Serialize)]
struct ErrorBody {
    success: bool,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

#[derive(Debug)]
pub enum ApiError {
    Domain(DomainError),
    Validation(ValidationErrors),
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        ApiError::Domain(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details): (StatusCode, &'static str, String, Option<Value>) =
            match self {
                ApiError::Domain(DomainError::UserNotFound) => {
                    (StatusCode::NOT_FOUND, "USER_NOT_FOUND", "User not found".into(), None)
                }
                ApiError::Domain(DomainError::EmailConflict) => {
                    (StatusCode::CONFLICT, "EMAIL_CONFLICT", "Email already in use".into(), None)
                }
                ApiError::Domain(DomainError::PhoneConflict) => {
                    (StatusCode::CONFLICT, "PHONE_CONFLICT", "Phone number already in use".into(), None)
                }
                ApiError::Domain(DomainError::InvalidCredentials) => {
                    (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS", "Invalid email or password".into(), None)
                }
                ApiError::Domain(DomainError::InvalidToken) => {
                    (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", "Invalid or expired token".into(), None)
                }
                ApiError::Domain(DomainError::AccountInactive) => {
                    (StatusCode::FORBIDDEN, "ACCOUNT_INACTIVE", "Account is inactive".into(), None)
                }
                ApiError::Domain(DomainError::Validation(msg)) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", msg, None)
                }
                ApiError::Domain(DomainError::Internal(msg)) => {
                    error!("Domain error: {msg}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "An unexpected error occurred".into(), None)
                }
                ApiError::Validation(errs) => {
                    let details = serde_json::to_value(&errs).ok();
                    (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", "Request validation failed".into(), details)
                }
                ApiError::Unauthorized(msg) => {
                    (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg, None)
                }
                ApiError::BadRequest(msg) => {
                    (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg, None)
                }
                ApiError::Internal(msg) => {
                    error!("Internal error: {msg}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "An unexpected error occurred".into(), None)
                }
            };

        (status, Json(ErrorBody { success: false, error: ErrorDetail { code, message, details } }))
            .into_response()
    }
}
