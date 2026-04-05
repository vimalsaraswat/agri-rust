use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MspRecord {
    pub id: String,
    pub crop: String,
    pub season: MspSeason,
    pub year: String,
    pub msp_per_quintal: u32,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum MspSeason {
    Kharif,
    Rabi,
}

impl MspSeason {
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "kharif" => Some(Self::Kharif),
            "rabi" => Some(Self::Rabi),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kharif => "Kharif",
            Self::Rabi => "Rabi",
        }
    }
}
