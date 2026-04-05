use crate::domain::{errors::DomainError, msp::MspSeason};

use super::{
    dto::{MspListResponse, MspPriceDto, MspQuery},
    repository::{MspFilter, MspRepository},
};

pub struct MspService<R: MspRepository> {
    repo: R,
}

impl<R: MspRepository> MspService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_prices(&self, query: MspQuery) -> Result<MspListResponse, DomainError> {
        let season = match &query.season {
            Some(s) => match MspSeason::from_str_ci(s) {
                Some(v) => Some(v),
                None => {
                    return Err(DomainError::Validation(
                        "Invalid season. Use 'kharif' or 'rabi'".into(),
                    ))
                }
            },
            None => None,
        };

        let filter = MspFilter {
            season,
            year: query.year,
            crop: query.crop,
        };

        let records = self.repo.find_all(&filter).await?;

        let prices: Vec<MspPriceDto> = records
            .into_iter()
            .map(|r| MspPriceDto {
                crop: r.crop,
                season: r.season.as_str().to_string(),
                year: r.year,
                msp_per_quintal: r.msp_per_quintal,
                unit: r.unit,
            })
            .collect();

        let total = prices.len();
        Ok(MspListResponse { total, prices })
    }
}
