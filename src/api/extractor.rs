use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    api::{error::ApiError, router::AppState},
    common::jwt,
    domain::user::UserRole,
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub role: UserRole,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                ApiError::Unauthorized("Missing or malformed Authorization header".into())
            })?
            .to_owned();

        let claims = jwt::decode_token(&token, &state.config.jwt_access_secret)
            .map_err(|_| ApiError::Unauthorized("Invalid or expired access token".into()))?;

        let role = match claims.role.as_str() {
            "Admin" => UserRole::Admin,
            _ => UserRole::Farmer,
        };

        Ok(AuthUser {
            user_id: claims.sub,
            role,
        })
    }
}
