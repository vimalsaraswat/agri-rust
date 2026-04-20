use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    Json,
};
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
        plant::dto::PlantDiagnoseResponse,
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

pub async fn diagnose_plant(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ApiResponse<PlantDiagnoseResponse>>), ApiError> {
    const MAX_SIZE: usize = 4 * 1024 * 1024; // 4MB

    let mut image_bytes: Option<Vec<u8>> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to read multipart field: {e}"))
    })? {
        if field.name() != Some("image") {
            continue;
        }

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        if !matches!(
            content_type.as_str(),
            "image/jpeg" | "image/png" | "image/webp" | "image/heic" | "image/heif"
        ) {
            return Err(ApiError::BadRequest(
                "Unsupported image type. Use JPEG, PNG, or WebP.".into(),
            ));
        }

        let bytes = field.bytes().await.map_err(|e| {
            ApiError::BadRequest(format!("Failed to read image data: {e}"))
        })?;

        if bytes.len() > MAX_SIZE {
            return Err(ApiError::BadRequest(
                "Image exceeds the 4MB size limit.".into(),
            ));
        }

        mime_type = Some(content_type);
        image_bytes = Some(bytes.to_vec());
        break;
    }

    let bytes = image_bytes
        .ok_or_else(|| ApiError::BadRequest("Missing 'image' field in form data.".into()))?;
    let mime = mime_type.unwrap();

    let result = state.plant_service.diagnose(&auth.user_id, bytes, mime).await?;
    Ok(ApiResponse::ok("Plant diagnosis complete", result))
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
