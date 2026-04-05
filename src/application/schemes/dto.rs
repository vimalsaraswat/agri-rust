use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SchemesQuery {
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SchemeDto {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub eligibility: String,
    pub benefits: String,
    pub apply_url: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct SchemesResponse {
    pub total: usize,
    pub schemes: Vec<SchemeDto>,
}

#[derive(Debug, Serialize)]
pub struct SchemeNewsDto {
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct NewsResponse {
    pub total: usize,
    pub cached_at: String,
    pub items: Vec<SchemeNewsDto>,
}
