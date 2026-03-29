use async_trait::async_trait;

use crate::domain::{
    errors::DomainError,
    user::{FarmInfo, Language, User, UserPreferences},
};

#[derive(Debug, Default)]
pub struct ProfileUpdate {
    pub full_name: Option<String>,
    pub preferred_language: Option<Language>,
    pub farm_info: Option<FarmInfo>,
    pub preferences: Option<UserPreferences>,
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, user: User) -> Result<String, DomainError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, DomainError>;
    async fn update_refresh_token(
        &self,
        id: &str,
        token_hash: Option<&str>,
    ) -> Result<(), DomainError>;
    async fn update_profile(&self, id: &str, update: &ProfileUpdate) -> Result<User, DomainError>;
    async fn email_exists(&self, email: &str) -> Result<bool, DomainError>;
    async fn phone_exists(&self, phone: &str) -> Result<bool, DomainError>;
}
