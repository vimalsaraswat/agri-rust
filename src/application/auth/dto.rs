use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::user::{
    FarmInfo, IrrigationMethod, Language, Season, SoilType, UserPreferences, UserRole, WaterSource,
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Full name must be between 2 and 100 characters"
    ))]
    pub full_name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(custom(function = "validate_phone_number"))]
    pub phone_number: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    #[validate(custom(function = "validate_password_complexity"))]
    pub password: String,

    pub preferred_language: Language,

    pub farm_info: Option<FarmInfoRequest>,

    pub preferences: Option<UserPreferencesRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Full name must be between 2 and 100 characters"
    ))]
    pub full_name: Option<String>,

    pub preferred_language: Option<Language>,
    pub farm_info: Option<FarmInfoRequest>,
    pub preferences: Option<UserPreferencesRequest>,
}

#[derive(Debug, Deserialize)]
pub struct FarmInfoRequest {
    pub irrigation_method: Option<IrrigationMethod>,
    pub water_sources: Option<Vec<WaterSource>>,
    pub soil_type: Option<SoilType>,
    #[serde(default)]
    pub land_area_acres: Option<f64>,
    pub preferred_seasons: Option<Vec<Season>>,
}

impl From<FarmInfoRequest> for FarmInfo {
    fn from(req: FarmInfoRequest) -> Self {
        FarmInfo {
            irrigation_method: req.irrigation_method,
            water_sources: req.water_sources.unwrap_or_default(),
            soil_type: req.soil_type,
            land_area_acres: req.land_area_acres,
            preferred_seasons: req.preferred_seasons.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserPreferencesRequest {
    pub water_savings_report: Option<bool>,
    pub weather_alerts: Option<bool>,
    pub expert_consultation: Option<bool>,
}

impl From<UserPreferencesRequest> for UserPreferences {
    fn from(req: UserPreferencesRequest) -> Self {
        UserPreferences {
            water_savings_report: req.water_savings_report.unwrap_or(true),
            weather_alerts: req.weather_alerts.unwrap_or(true),
            expert_consultation: req.expert_consultation.unwrap_or(false),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub preferred_language: Language,
    pub farm_info: FarmInfo,
    pub preferences: UserPreferences,
    pub role: UserRole,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::domain::user::User> for UserProfile {
    fn from(user: crate::domain::user::User) -> Self {
        UserProfile {
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            phone_number: user.phone_number,
            preferred_language: user.preferred_language,
            farm_info: user.farm_info,
            preferences: user.preferences,
            role: user.role,
            is_active: user.is_active,
            is_email_verified: user.is_email_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub tokens: AuthTokens,
}

use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\+91|91|0)?[6-9]\d{9}$").expect("Invalid phone regex"));

fn validate_phone_number(phone: &str) -> Result<(), validator::ValidationError> {
    if PHONE_REGEX.is_match(phone) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_phone");
        err.message = Some(
            "Phone number must be a valid Indian mobile number (e.g. +919876543210 or 9876543210)"
                .into(),
        );
        Err(err)
    }
}

fn validate_password_complexity(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;':\",./<>?".contains(c));

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("password_complexity");
        err.message = Some(
            "Password must contain at least one uppercase letter, one lowercase letter, \
             one digit, and one special character"
                .into(),
        );
        Err(err)
    }
}
