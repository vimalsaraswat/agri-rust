use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    Hindi,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Marathi,
    Punjabi,
    Bengali,
    Gujarati,
    Odia,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrrigationMethod {
    Drip,
    Sprinkler,
    FloodFurrow,
    Surface,
    SubSurface,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaterSource {
    Borewell,
    Canal,
    River,
    Rainwater,
    Pond,
    DugWell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SoilType {
    Alluvial,
    Black,
    Red,
    Laterite,
    Mountain,
    Desert,
    Peaty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Season {
    Kharif,
    Rabi,
    Zaid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum UserRole {
    #[default]
    Farmer,
    Admin,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FarmInfo {
    pub irrigation_method: Option<IrrigationMethod>,
    pub water_sources: Vec<WaterSource>,
    pub soil_type: Option<SoilType>,
    pub land_area_acres: Option<f64>,
    pub preferred_seasons: Vec<Season>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub water_savings_report: bool,
    pub weather_alerts: bool,
    pub expert_consultation: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            water_savings_report: true,
            weather_alerts: true,
            expert_consultation: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub password_hash: String,
    pub preferred_language: Language,
    pub farm_info: FarmInfo,
    pub preferences: UserPreferences,
    pub role: UserRole,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub refresh_token_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        full_name: String,
        email: String,
        phone_number: String,
        password_hash: String,
        preferred_language: Language,
        farm_info: FarmInfo,
        preferences: UserPreferences,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            full_name,
            email,
            phone_number,
            password_hash,
            preferred_language,
            farm_info,
            preferences,
            role: UserRole::Farmer,
            is_active: true,
            is_email_verified: false,
            refresh_token_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
}
