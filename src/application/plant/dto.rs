use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlantDiagnoseResponse {
    pub health_status: String,
    pub confidence: String,
    pub plant_identified: String,
    pub disease_name: Option<String>,
    pub symptoms: Vec<String>,
    pub affected_parts: Vec<String>,
    pub treatment: Option<String>,
    pub prevention: String,
    pub advisory: String,
}
