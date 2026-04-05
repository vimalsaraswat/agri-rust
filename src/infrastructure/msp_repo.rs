use async_trait::async_trait;
use mongodb::{
    bson::{doc, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    application::msp::repository::{MspFilter, MspRepository},
    domain::{
        errors::DomainError,
        msp::{MspRecord, MspSeason},
    },
};

const SEED_JSON: &str = include_str!("../../data/msp_seed.json");

#[derive(Debug, Serialize, Deserialize)]
struct MspDocument {
    #[serde(rename = "_id")]
    id: String,
    crop: String,
    season: MspSeason,
    year: String,
    msp_per_quintal: u32,
    unit: String,
}

#[derive(Deserialize)]
struct SeedRecord {
    crop: String,
    season: MspSeason,
    year: String,
    msp_per_quintal: u32,
    unit: String,
}

impl From<MspDocument> for MspRecord {
    fn from(d: MspDocument) -> Self {
        Self {
            id: d.id,
            crop: d.crop,
            season: d.season,
            year: d.year,
            msp_per_quintal: d.msp_per_quintal,
            unit: d.unit,
        }
    }
}

pub struct MongoMspRepository {
    col: Collection<MspDocument>,
}

impl MongoMspRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            col: db.collection("msp_prices"),
        }
    }

    pub async fn ensure_ready(&self) {
        self.create_indexes().await;
        self.seed_if_empty().await;
    }

    async fn create_indexes(&self) {
        let col: Collection<Document> = self.col.clone_with_type();
        let index = IndexModel::builder()
            .keys(doc! { "season": 1, "year": 1, "crop": 1 })
            .options(IndexOptions::builder().build())
            .build();

        if let Err(e) = col.create_index(index).await {
            warn!("MSP index creation failed: {e}");
        }
    }

    async fn seed_if_empty(&self) {
        let count = self.col.count_documents(doc! {}).await.unwrap_or(1);
        if count > 0 {
            return;
        }

        let records: Vec<SeedRecord> = match serde_json::from_str(SEED_JSON) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse MSP seed data: {e}");
                return;
            }
        };

        let docs: Vec<MspDocument> = records
            .into_iter()
            .map(|r| MspDocument {
                id: Uuid::new_v4().to_string(),
                crop: r.crop,
                season: r.season,
                year: r.year,
                msp_per_quintal: r.msp_per_quintal,
                unit: r.unit,
            })
            .collect();

        let n = docs.len();
        match self.col.insert_many(docs).await {
            Ok(_) => info!("Seeded {n} MSP price records"),
            Err(e) => warn!("MSP seeding failed: {e}"),
        }
    }
}

#[async_trait]
impl MspRepository for MongoMspRepository {
    async fn find_all(&self, filter: &MspFilter) -> Result<Vec<MspRecord>, DomainError> {
        let mut query = doc! {};

        if let Some(season) = &filter.season {
            query.insert("season", season.as_str());
        }
        if let Some(year) = &filter.year {
            query.insert("year", year.as_str());
        }
        if let Some(crop) = &filter.crop {
            query.insert("crop", doc! { "$regex": crop, "$options": "i" });
        }

        let mut cursor = self
            .col
            .find(query)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut records = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            records.push(MspRecord::from(doc));
        }

        Ok(records)
    }
}
