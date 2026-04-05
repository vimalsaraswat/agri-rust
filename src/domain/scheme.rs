use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub eligibility: String,
    pub benefits: String,
    pub apply_url: String,
    pub category: SchemeCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SchemeCategory {
    IncomeSupport,
    Insurance,
    Credit,
    Irrigation,
    MarketAccess,
    Infrastructure,
    Training,
}

impl SchemeCategory {
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('_', "").as_str() {
            "incomesupport" => Some(Self::IncomeSupport),
            "insurance" => Some(Self::Insurance),
            "credit" => Some(Self::Credit),
            "irrigation" => Some(Self::Irrigation),
            "marketaccess" => Some(Self::MarketAccess),
            "infrastructure" => Some(Self::Infrastructure),
            "training" => Some(Self::Training),
            _ => None,
        }
    }

    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::IncomeSupport => "income_support",
            Self::Insurance => "insurance",
            Self::Credit => "credit",
            Self::Irrigation => "irrigation",
            Self::MarketAccess => "market_access",
            Self::Infrastructure => "infrastructure",
            Self::Training => "training",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeNews {
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at: String,
    pub description: String,
}
