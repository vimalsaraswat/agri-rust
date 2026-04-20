use std::sync::Arc;

use crate::{
    application::repository::UserRepository,
    domain::{
        errors::DomainError,
        user::{Language, User},
    },
    infrastructure::GeminiClient,
};

use super::dto::PlantDiagnoseResponse;

pub struct PlantService<R: UserRepository> {
    repo: Arc<R>,
    gemini: GeminiClient,
}

impl<R: UserRepository> PlantService<R> {
    pub fn new(repo: Arc<R>, gemini: GeminiClient) -> Self {
        Self { repo, gemini }
    }

    pub async fn diagnose(
        &self,
        user_id: &str,
        image_bytes: Vec<u8>,
        mime_type: String,
    ) -> Result<PlantDiagnoseResponse, DomainError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::UserNotFound)?;

        if !user.is_active {
            return Err(DomainError::AccountInactive);
        }

        let system_prompt = build_system_prompt(&user);
        let user_text = build_user_text(&user);

        let raw = self
            .gemini
            .generate_with_image(&system_prompt, &user_text, image_bytes, &mime_type)
            .await?;

        serde_json::from_str::<PlantDiagnoseResponse>(&raw).map_err(|e| {
            tracing::error!("Failed to parse plant diagnosis JSON: {e}\nRaw: {raw}");
            DomainError::Internal("Failed to parse diagnosis response".into())
        })
    }
}

fn build_system_prompt(user: &User) -> String {
    let lang_instruction = lang_prompt(&user.preferred_language);

    format!(
        r#"You are an expert plant pathologist and agricultural advisor helping Indian farmers diagnose plant diseases.

{lang_instruction}

Analyse the provided image and respond with ONLY a valid JSON object (no markdown, no code fences) matching exactly this schema:
{{
  "health_status": "Healthy" | "Diseased" | "Unclear",
  "confidence": "High" | "Medium" | "Low",
  "plant_identified": "<plant name or 'Unknown'>",
  "disease_name": "<disease name or null if healthy>",
  "symptoms": ["<symptom 1>", "<symptom 2>"],
  "affected_parts": ["<part 1>", "<part 2>"],
  "treatment": "<treatment steps or null if healthy>",
  "prevention": "<prevention advice>",
  "advisory": "<friendly overall advice in the farmer's language>"
}}

Rules:
- If the image is not a plant, set health_status to "Unclear" and explain in advisory.
- Keep symptoms and affected_parts as concise arrays.
- The advisory field MUST be in the farmer's preferred language using native script.
- All other fields should be in English."#,
        lang_instruction = lang_instruction,
    )
}

fn build_user_text(user: &User) -> String {
    format!(
        "Please diagnose this plant image for farmer {}. Provide the response in {} as instructed.",
        user.full_name,
        language_display(&user.preferred_language),
    )
}

fn lang_prompt(lang: &Language) -> &'static str {
    match lang {
        Language::English => "The advisory field must be in English.",
        Language::Hindi => "The advisory field must be in Hindi using Devanagari script (हिंदी).",
        Language::Tamil => "The advisory field must be in Tamil using Tamil script (தமிழ்).",
        Language::Telugu => "The advisory field must be in Telugu using Telugu script (తెలుగు).",
        Language::Kannada => "The advisory field must be in Kannada using Kannada script (ಕನ್ನಡ).",
        Language::Malayalam => "The advisory field must be in Malayalam using Malayalam script (മലയാളം).",
        Language::Marathi => "The advisory field must be in Marathi using Devanagari script (मराठी).",
        Language::Punjabi => "The advisory field must be in Punjabi using Gurmukhi script (ਪੰਜਾਬੀ).",
        Language::Bengali => "The advisory field must be in Bengali using Bengali script (বাংলা).",
        Language::Gujarati => "The advisory field must be in Gujarati using Gujarati script (ગુજરાતી).",
        Language::Odia => "The advisory field must be in Odia using Odia script (ଓଡ଼ିଆ).",
    }
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
