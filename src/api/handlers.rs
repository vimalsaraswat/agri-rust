use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use validator::Validate;

use crate::{
    api::{
        error::ApiError,
        extractor::AuthUser,
        response::ApiResponse,
        router::AppState,
    },
    application::auth::dto::{
        AuthResponse, AuthTokens, LoginRequest, RefreshTokenRequest, RegisterRequest,
        UpdateProfileRequest, UserProfile,
    },
};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthResponse>>), ApiError> {
    body.validate().map_err(ApiError::Validation)?;
    let result = state.auth_service.register(body).await?;
    Ok(ApiResponse::created("Registration successful", result))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthResponse>>), ApiError> {
    body.validate().map_err(ApiError::Validation)?;
    let result = state.auth_service.login(body).await?;
    Ok(ApiResponse::ok("Login successful", result))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthTokens>>), ApiError> {
    body.validate().map_err(ApiError::Validation)?;
    let tokens = state.auth_service.refresh_tokens(body).await?;
    Ok(ApiResponse::ok("Token refreshed", tokens))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    state.auth_service.logout(&auth.user_id).await?;
    Ok(ApiResponse::message("Logged out successfully"))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), ApiError> {
    let profile = state.auth_service.get_me(&auth.user_id).await?;
    Ok(ApiResponse::ok("Profile retrieved", profile))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), ApiError> {
    body.validate().map_err(ApiError::Validation)?;
    let profile = state.auth_service.update_me(&auth.user_id, body).await?;
    Ok(ApiResponse::ok("Profile updated", profile))
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

pub async fn health() -> (StatusCode, Json<ApiResponse<HealthResponse>>) {
    ApiResponse::ok("Service is healthy", HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}
