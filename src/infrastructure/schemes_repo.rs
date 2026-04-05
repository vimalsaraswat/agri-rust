use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, to_bson, DateTime as BsonDateTime},
    options::ReplaceOptions,
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::{
    errors::DomainError,
    scheme::{Scheme, SchemeCategory, SchemeNews},
};

const SEED_JSON: &str = include_str!("../../data/schemes_seed.json");

#[derive(Debug, Serialize, Deserialize)]
struct SchemeDocument {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    short_name: String,
    description: String,
    eligibility: String,
    benefits: String,
    apply_url: String,
    category: SchemeCategory,
}

#[derive(Deserialize)]
struct SchemeSeedRecord {
    name: String,
    short_name: String,
    description: String,
    eligibility: String,
    benefits: String,
    apply_url: String,
    category: SchemeCategory,
}

impl From<SchemeDocument> for Scheme {
    fn from(d: SchemeDocument) -> Self {
        Self {
            id: d.id,
            name: d.name,
            short_name: d.short_name,
            description: d.description,
            eligibility: d.eligibility,
            benefits: d.benefits,
            apply_url: d.apply_url,
            category: d.category,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NewsItemDocument {
    title: String,
    url: String,
    source: String,
    published_at: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewsCacheDocument {
    #[serde(rename = "_id")]
    id: String,
    items: Vec<NewsItemDocument>,
    fetched_at: BsonDateTime,
}

#[derive(Clone)]
pub struct MongoSchemesRepo {
    schemes_col: Collection<SchemeDocument>,
    news_col: Collection<NewsCacheDocument>,
}

impl MongoSchemesRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            schemes_col: db.collection("schemes"),
            news_col: db.collection("schemes_news_cache"),
        }
    }

    pub async fn ensure_ready(&self) {
        self.seed_if_empty().await;
    }

    async fn seed_if_empty(&self) {
        let count = self.schemes_col.count_documents(doc! {}).await.unwrap_or(1);
        if count > 0 {
            return;
        }

        let records: Vec<SchemeSeedRecord> = match serde_json::from_str(SEED_JSON) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse schemes seed: {e}");
                return;
            }
        };

        let docs: Vec<SchemeDocument> = records
            .into_iter()
            .map(|r| SchemeDocument {
                id: Uuid::new_v4().to_string(),
                name: r.name,
                short_name: r.short_name,
                description: r.description,
                eligibility: r.eligibility,
                benefits: r.benefits,
                apply_url: r.apply_url,
                category: r.category,
            })
            .collect();

        let n = docs.len();
        match self.schemes_col.insert_many(docs).await {
            Ok(_) => info!("Seeded {n} government schemes"),
            Err(e) => warn!("Schemes seeding failed: {e}"),
        }
    }

    pub async fn get_schemes(
        &self,
        category: Option<&SchemeCategory>,
    ) -> Result<Vec<Scheme>, DomainError> {
        let filter = if let Some(cat) = category {
            let cat_bson = to_bson(cat).map_err(|e| DomainError::Internal(e.to_string()))?;
            doc! { "category": cat_bson }
        } else {
            doc! {}
        };

        let mut cursor = self
            .schemes_col
            .find(filter)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut schemes = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            schemes.push(Scheme::from(doc));
        }

        Ok(schemes)
    }

    pub async fn get_cached_news(
        &self,
    ) -> Result<Option<(Vec<SchemeNews>, DateTime<Utc>)>, DomainError> {
        let doc = self
            .news_col
            .find_one(doc! { "_id": "news_cache" })
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(doc.map(|d| {
            let fetched_at = DateTime::from_timestamp_millis(d.fetched_at.timestamp_millis())
                .unwrap_or_else(Utc::now);
            let items = d
                .items
                .into_iter()
                .map(|i| SchemeNews {
                    title: i.title,
                    url: i.url,
                    source: i.source,
                    published_at: i.published_at,
                    description: i.description,
                })
                .collect();
            (items, fetched_at)
        }))
    }

    pub async fn set_cached_news(
        &self,
        items: &[SchemeNews],
        fetched_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let doc_items: Vec<NewsItemDocument> = items
            .iter()
            .map(|n| NewsItemDocument {
                title: n.title.clone(),
                url: n.url.clone(),
                source: n.source.clone(),
                published_at: n.published_at.clone(),
                description: n.description.clone(),
            })
            .collect();

        let cache_doc = NewsCacheDocument {
            id: "news_cache".to_string(),
            items: doc_items,
            fetched_at: BsonDateTime::from_millis(fetched_at.timestamp_millis()),
        };

        self.news_col
            .replace_one(doc! { "_id": "news_cache" }, cache_doc)
            .with_options(ReplaceOptions::builder().upsert(true).build())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
