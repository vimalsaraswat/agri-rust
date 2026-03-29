use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, serde_helpers::chrono_datetime_as_bson_datetime, to_bson, Document},
    Collection,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    application::repository::{ProfileUpdate, UserRepository},
    domain::{
        errors::DomainError,
        user::{FarmInfo, Language, User, UserPreferences, UserRole},
    },
};

#[derive(Debug, Serialize, Deserialize)]
struct UserDocument {
    #[serde(rename = "_id")]
    id: String,
    full_name: String,
    email: String,
    phone_number: String,
    password_hash: String,
    preferred_language: Language,
    farm_info: FarmInfo,
    preferences: UserPreferences,
    role: UserRole,
    is_active: bool,
    is_email_verified: bool,
    refresh_token_hash: Option<String>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    created_at: DateTime<Utc>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    updated_at: DateTime<Utc>,
}

impl From<User> for UserDocument {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            full_name: u.full_name,
            email: u.email,
            phone_number: u.phone_number,
            password_hash: u.password_hash,
            preferred_language: u.preferred_language,
            farm_info: u.farm_info,
            preferences: u.preferences,
            role: u.role,
            is_active: u.is_active,
            is_email_verified: u.is_email_verified,
            refresh_token_hash: u.refresh_token_hash,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

impl From<UserDocument> for User {
    fn from(d: UserDocument) -> Self {
        Self {
            id: d.id,
            full_name: d.full_name,
            email: d.email,
            phone_number: d.phone_number,
            password_hash: d.password_hash,
            preferred_language: d.preferred_language,
            farm_info: d.farm_info,
            preferences: d.preferences,
            role: d.role,
            is_active: d.is_active,
            is_email_verified: d.is_email_verified,
            refresh_token_hash: d.refresh_token_hash,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

pub struct MongoUserRepository {
    col: Collection<UserDocument>,
}

impl MongoUserRepository {
    pub fn new(db: &mongodb::Database) -> Self {
        Self {
            col: db.collection("users"),
        }
    }
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn create(&self, user: User) -> Result<String, DomainError> {
        let id = user.id.clone();
        self.col
            .insert_one(UserDocument::from(user))
            .await
            .map_err(|e| db_error("insert", e))?;
        Ok(id)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        self.col
            .find_one(doc! { "_id": id })
            .await
            .map(|opt| opt.map(User::from))
            .map_err(|e| db_error("find_by_id", e))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        self.col
            .find_one(doc! { "email": email })
            .await
            .map(|opt| opt.map(User::from))
            .map_err(|e| db_error("find_by_email", e))
    }

    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, DomainError> {
        self.col
            .find_one(doc! { "phone_number": phone })
            .await
            .map(|opt| opt.map(User::from))
            .map_err(|e| db_error("find_by_phone", e))
    }

    async fn update_refresh_token(
        &self,
        id: &str,
        token_hash: Option<&str>,
    ) -> Result<(), DomainError> {
        let update = match token_hash {
            Some(hash) => doc! { "$set": { "refresh_token_hash": hash, "updated_at": now_bson() } },
            None => {
                doc! { "$unset": { "refresh_token_hash": "" }, "$set": { "updated_at": now_bson() } }
            }
        };
        self.col
            .update_one(doc! { "_id": id }, update)
            .await
            .map(|_| ())
            .map_err(|e| db_error("update_refresh_token", e))
    }

    async fn update_profile(&self, id: &str, update: &ProfileUpdate) -> Result<User, DomainError> {
        let mut set = Document::new();
        set.insert("updated_at", now_bson());

        if let Some(ref name) = update.full_name {
            set.insert("full_name", name.as_str());
        }
        if let Some(ref lang) = update.preferred_language {
            set.insert("preferred_language", to_bson(lang).map_err(ser_err)?);
        }
        if let Some(ref fi) = update.farm_info {
            set.insert("farm_info", to_bson(fi).map_err(ser_err)?);
        }
        if let Some(ref prefs) = update.preferences {
            set.insert("preferences", to_bson(prefs).map_err(ser_err)?);
        }

        let opts = mongodb::options::FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        self.col
            .find_one_and_update(doc! { "_id": id }, doc! { "$set": set })
            .with_options(opts)
            .await
            .map_err(|e| db_error("update_profile", e))?
            .map(User::from)
            .ok_or(DomainError::UserNotFound)
    }

    async fn email_exists(&self, email: &str) -> Result<bool, DomainError> {
        self.col
            .count_documents(doc! { "email": email })
            .await
            .map(|n| n > 0)
            .map_err(|e| db_error("email_exists", e))
    }

    async fn phone_exists(&self, phone: &str) -> Result<bool, DomainError> {
        self.col
            .count_documents(doc! { "phone_number": phone })
            .await
            .map(|n| n > 0)
            .map_err(|e| db_error("phone_exists", e))
    }
}

fn db_error(op: &str, e: mongodb::error::Error) -> DomainError {
    error!("MongoDB {op} error: {e}");
    DomainError::Internal(format!("Database error in {op}"))
}

fn ser_err(e: mongodb::bson::ser::Error) -> DomainError {
    DomainError::Internal(format!("Serialisation error: {e}"))
}

fn now_bson() -> mongodb::bson::DateTime {
    mongodb::bson::DateTime::now()
}
