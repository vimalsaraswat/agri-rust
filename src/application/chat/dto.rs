use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ChatRequest {
    #[validate(length(min = 1, max = 2000, message = "Message must be 1–2000 characters"))]
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponseDto {
    pub reply: String,
}
