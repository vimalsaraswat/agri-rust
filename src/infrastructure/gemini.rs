use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::domain::errors::DomainError;

const GEMINI_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite-preview:generateContent";

#[derive(Serialize)]
struct GeminiRequest<'a> {
    system_instruction: SystemInstruction<'a>,
    contents: Vec<Content<'a>>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct SystemInstruction<'a> {
    parts: [Part<'a>; 1],
}

#[derive(Serialize)]
struct Content<'a> {
    role: &'a str,
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Clone)]
pub struct GeminiClient {
    http: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
        }
    }

    pub async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, DomainError> {
        let body = GeminiRequest {
            system_instruction: SystemInstruction {
                parts: [Part {
                    text: system_prompt,
                }],
            },
            contents: vec![Content {
                role: "user",
                parts: vec![Part { text: user_message }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: 1024,
                temperature: 0.4,
            },
        };

        let resp = self
            .http
            .post(GEMINI_URL)
            .query(&[("key", &self.api_key)])
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Gemini request failed: {e}");
                DomainError::Internal("AI service unavailable".into())
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            error!("Gemini error {status}: {text}");
            return Err(DomainError::Internal("AI service returned an error".into()));
        }

        let parsed: GeminiResponse = resp.json().await.map_err(|e| {
            error!("Gemini response parse error: {e}");
            DomainError::Internal("Failed to parse AI response".into())
        })?;

        parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| DomainError::Internal("Empty AI response".into()))
    }
}
