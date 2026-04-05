use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MspQuery {
    pub season: Option<String>,
    pub year: Option<String>,
    pub crop: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MspPriceDto {
    pub crop: String,
    pub season: String,
    pub year: String,
    pub msp_per_quintal: u32,
    pub unit: String,
}

#[derive(Debug, Serialize)]
pub struct MspListResponse {
    pub total: usize,
    pub prices: Vec<MspPriceDto>,
}
