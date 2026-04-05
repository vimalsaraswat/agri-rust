use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Serialize;
use validator::Validate;

use crate::{
    api::{
        error::ApiError,
        extractor::AuthUser,
        response::ApiResponse,
        router::AppState,
    },
    application::{
        auth::dto::{
            AuthResponse, AuthTokens, LoginRequest, RefreshTokenRequest, RegisterRequest,
            UpdateProfileRequest, UserProfile,
        },
        chat::dto::{ChatRequest, ChatResponseDto},
        msp::dto::{MspListResponse, MspQuery},
        schemes::dto::{NewsResponse, SchemesQuery, SchemesResponse},
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

pub async fn chat(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ChatResponseDto>>), ApiError> {
    body.validate().map_err(ApiError::Validation)?;
    let result = state.chat_service.chat(&auth.user_id, body).await?;
    Ok(ApiResponse::ok("Response generated", result))
}

pub async fn schemes(
    State(state): State<AppState>,
    Query(params): Query<SchemesQuery>,
) -> Result<(StatusCode, Json<ApiResponse<SchemesResponse>>), ApiError> {
    let result = state.schemes_service.get_schemes(params).await?;
    Ok(ApiResponse::ok("Schemes fetched", result))
}

pub async fn schemes_news(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<NewsResponse>>), ApiError> {
    let result = state.schemes_service.get_news().await?;
    Ok(ApiResponse::ok("Latest scheme news fetched", result))
}

pub async fn msp_prices(
    State(state): State<AppState>,
    Query(params): Query<MspQuery>,
) -> Result<(StatusCode, Json<ApiResponse<MspListResponse>>), ApiError> {
    let result = state.msp_service.get_prices(params).await?;
    Ok(ApiResponse::ok("MSP prices fetched", result))
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
