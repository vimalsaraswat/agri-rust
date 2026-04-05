use async_trait::async_trait;

use crate::domain::{
    errors::DomainError,
    msp::{MspRecord, MspSeason},
};

pub struct MspFilter {
    pub season: Option<MspSeason>,
    pub year: Option<String>,
    pub crop: Option<String>,
}

#[async_trait]
pub trait MspRepository: Send + Sync {
    async fn find_all(&self, filter: &MspFilter) -> Result<Vec<MspRecord>, DomainError>;
}
