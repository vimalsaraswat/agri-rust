use std::sync::Arc;

use crate::{
    application::repository::UserRepository,
    domain::{
        errors::DomainError,
        user::{FarmInfo, IrrigationMethod, Language, Season, SoilType, User, WaterSource},
    },
    infrastructure::GeminiClient,
};

use super::dto::{ChatRequest, ChatResponseDto};

pub struct ChatService<R: UserRepository> {
    repo: Arc<R>,
    gemini: GeminiClient,
}

impl<R: UserRepository> ChatService<R> {
    pub fn new(repo: Arc<R>, gemini: GeminiClient) -> Self {
        Self { repo, gemini }
    }

    pub async fn chat(
        &self,
        user_id: &str,
        req: ChatRequest,
    ) -> Result<ChatResponseDto, DomainError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::UserNotFound)?;

        if !user.is_active {
            return Err(DomainError::AccountInactive);
        }

        let system_prompt = build_system_prompt(&user);
        let reply = self.gemini.generate(&system_prompt, &req.message).await?;

        Ok(ChatResponseDto { reply })
    }
}

fn build_system_prompt(user: &User) -> String {
    let language_instruction = language_prompt(&user.preferred_language);
    let farm_summary = farm_summary(&user.farm_info);

    format!(
        r#"You are Kisaan Sahayak, a knowledgeable and friendly agricultural expert helping Indian farmers.

Farmer Profile:
- Name: {name}
- Preferred Language: {lang_name}
{farm_summary}

Instructions:
- {lang_instruction}
- Give practical, actionable advice tailored to this farmer's specific soil type, irrigation method, and growing conditions.
- Keep responses clear and easy to understand for a rural farmer — avoid unnecessary jargon.
- Only answer questions related to agriculture, crop management, soil health, pest control, fertilisers, irrigation, weather, or government agricultural schemes (like MSP, PM-KISAN, crop insurance).
- If asked about anything unrelated to farming or agriculture, politely decline and redirect.
- Be warm, respectful, and encouraging."#,
        name = user.full_name,
        lang_name = language_display(&user.preferred_language),
        farm_summary = farm_summary,
        lang_instruction = language_instruction,
    )
}

fn language_display(lang: &Language) -> &'static str {
    match lang {
        Language::English => "English",
        Language::Hindi => "Hindi",
        Language::Tamil => "Tamil",
        Language::Telugu => "Telugu",
        Language::Kannada => "Kannada",
        Language::Malayalam => "Malayalam",
        Language::Marathi => "Marathi",
        Language::Punjabi => "Punjabi",
        Language::Bengali => "Bengali",
        Language::Gujarati => "Gujarati",
        Language::Odia => "Odia",
    }
}

fn language_prompt(lang: &Language) -> &'static str {
    match lang {
        Language::English => "Always respond in English.",
        Language::Hindi => "Always respond in Hindi using Devanagari script (हिंदी).",
        Language::Tamil => "Always respond in Tamil using Tamil script (தமிழ்).",
        Language::Telugu => "Always respond in Telugu using Telugu script (తెలుగు).",
        Language::Kannada => "Always respond in Kannada using Kannada script (ಕನ್ನಡ).",
        Language::Malayalam => "Always respond in Malayalam using Malayalam script (മലയാളം).",
        Language::Marathi => "Always respond in Marathi using Devanagari script (मराठी).",
        Language::Punjabi => "Always respond in Punjabi using Gurmukhi script (ਪੰਜਾਬੀ).",
        Language::Bengali => "Always respond in Bengali using Bengali script (বাংলা).",
        Language::Gujarati => "Always respond in Gujarati using Gujarati script (ગુજરાતી).",
        Language::Odia => "Always respond in Odia using Odia script (ଓଡ଼ିଆ).",
    }
}

fn farm_summary(farm: &FarmInfo) -> String {
    let mut parts = Vec::new();

    if let Some(soil) = &farm.soil_type {
        parts.push(format!("- Soil Type: {}", soil_display(soil)));
    }
    if let Some(method) = &farm.irrigation_method {
        parts.push(format!("- Irrigation Method: {}", irrigation_display(method)));
    }
    if !farm.water_sources.is_empty() {
        let sources: Vec<&str> = farm.water_sources.iter().map(water_source_display).collect();
        parts.push(format!("- Water Sources: {}", sources.join(", ")));
    }
    if let Some(area) = farm.land_area_acres {
        parts.push(format!("- Land Area: {area} acres"));
    }
    if !farm.preferred_seasons.is_empty() {
        let seasons: Vec<&str> = farm.preferred_seasons.iter().map(season_display).collect();
        parts.push(format!("- Preferred Seasons: {}", seasons.join(", ")));
    }

    if parts.is_empty() {
        "- Farm details: not provided".to_string()
    } else {
        parts.join("\n")
    }
}

fn soil_display(s: &SoilType) -> &'static str {
    match s {
        SoilType::Alluvial => "Alluvial",
        SoilType::Black => "Black (Regur)",
        SoilType::Red => "Red",
        SoilType::Laterite => "Laterite",
        SoilType::Mountain => "Mountain",
        SoilType::Desert => "Desert (Arid)",
        SoilType::Peaty => "Peaty",
    }
}

fn irrigation_display(m: &IrrigationMethod) -> &'static str {
    match m {
        IrrigationMethod::Drip => "Drip Irrigation",
        IrrigationMethod::Sprinkler => "Sprinkler",
        IrrigationMethod::FloodFurrow => "Flood/Furrow",
        IrrigationMethod::Surface => "Surface",
        IrrigationMethod::SubSurface => "Sub-surface",
        IrrigationMethod::Manual => "Manual",
    }
}

fn water_source_display(w: &WaterSource) -> &'static str {
    match w {
        WaterSource::Borewell => "Borewell",
        WaterSource::Canal => "Canal",
        WaterSource::River => "River",
        WaterSource::Rainwater => "Rainwater",
        WaterSource::Pond => "Pond",
        WaterSource::DugWell => "Dug Well",
    }
}

fn season_display(s: &Season) -> &'static str {
    match s {
        Season::Kharif => "Kharif",
        Season::Rabi => "Rabi",
        Season::Zaid => "Zaid",
    }
}
