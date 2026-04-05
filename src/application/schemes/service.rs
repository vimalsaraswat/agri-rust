use chrono::Utc;

use crate::domain::{errors::DomainError, scheme::SchemeCategory};
use crate::infrastructure::{MongoSchemesRepo, NewsFetcher};

use super::dto::{NewsResponse, SchemeDto, SchemeNewsDto, SchemesQuery, SchemesResponse};

const NEWS_CACHE_TTL_HOURS: i64 = 6;

pub struct SchemesService {
    repo: MongoSchemesRepo,
    fetcher: NewsFetcher,
}

impl SchemesService {
    pub fn new(repo: MongoSchemesRepo, fetcher: NewsFetcher) -> Self {
        Self { repo, fetcher }
    }

    pub async fn get_schemes(&self, query: SchemesQuery) -> Result<SchemesResponse, DomainError> {
        let category = match &query.category {
            Some(s) => match SchemeCategory::from_str_ci(s) {
                Some(c) => Some(c),
                None => {
                    return Err(DomainError::Validation(
                        "Invalid category. Valid values: income_support, insurance, credit, irrigation, market_access, infrastructure, training".into(),
                    ))
                }
            },
            None => None,
        };

        let schemes = self.repo.get_schemes(category.as_ref()).await?;

        let dtos: Vec<SchemeDto> = schemes
            .into_iter()
            .map(|s| SchemeDto {
                id: s.id,
                name: s.name,
                short_name: s.short_name,
                description: s.description,
                eligibility: s.eligibility,
                benefits: s.benefits,
                apply_url: s.apply_url,
                category: s.category.as_api_str().to_string(),
            })
            .collect();

        let total = dtos.len();
        Ok(SchemesResponse {
            total,
            schemes: dtos,
        })
    }

    pub async fn get_news(&self) -> Result<NewsResponse, DomainError> {
        if let Some((cached_items, fetched_at)) = self.repo.get_cached_news().await? {
            let age_hours = (Utc::now() - fetched_at).num_hours();
            if age_hours < NEWS_CACHE_TTL_HOURS {
                let items = cached_items
                    .into_iter()
                    .map(|n| SchemeNewsDto {
                        title: n.title,
                        url: n.url,
                        source: n.source,
                        published_at: n.published_at,
                        description: n.description,
                    })
                    .collect::<Vec<_>>();
                let total = items.len();
                return Ok(NewsResponse {
                    total,
                    cached_at: fetched_at.to_rfc3339(),
                    items,
                });
            }
        }

        let news = self.fetcher.fetch_schemes_news().await?;
        let now = Utc::now();
        self.repo.set_cached_news(&news, now).await?;

        let items = news
            .into_iter()
            .map(|n| SchemeNewsDto {
                title: n.title,
                url: n.url,
                source: n.source,
                published_at: n.published_at,
                description: n.description,
            })
            .collect::<Vec<_>>();

        let total = items.len();
        Ok(NewsResponse {
            total,
            cached_at: now.to_rfc3339(),
            items,
        })
    }
}
