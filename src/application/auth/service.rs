use std::sync::Arc;

use crate::{
    application::{
        auth::dto::{
            AuthResponse, AuthTokens, LoginRequest, RefreshTokenRequest, RegisterRequest,
            UpdateProfileRequest, UserProfile,
        },
        repository::{ProfileUpdate, UserRepository},
    },
    common::{crypto, jwt},
    config::AppConfig,
    domain::{
        errors::DomainError,
        user::{FarmInfo, User, UserPreferences},
    },
};

pub struct AuthService<R: UserRepository> {
    repo: Arc<R>,
    config: Arc<AppConfig>,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repo: Arc<R>, config: Arc<AppConfig>) -> Self {
        Self { repo, config }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, DomainError> {
        let email = req.email.trim().to_lowercase();
        let phone = normalise_phone(&req.phone_number);

        let (email_taken, phone_taken) =
            tokio::try_join!(self.repo.email_exists(&email), self.repo.phone_exists(&phone))?;

        if email_taken {
            return Err(DomainError::EmailConflict);
        }
        if phone_taken {
            return Err(DomainError::PhoneConflict);
        }

        let user = User::new(
            req.full_name.trim().to_string(),
            email,
            phone,
            crypto::hash(&req.password)?,
            req.preferred_language,
            req.farm_info.map(FarmInfo::from).unwrap_or_default(),
            req.preferences.map(UserPreferences::from).unwrap_or_default(),
        );

        let user_id = self.repo.create(user).await?;
        let user = self
            .repo
            .find_by_id(&user_id)
            .await?
            .ok_or_else(|| DomainError::Internal("User disappeared after insert".into()))?;

        let tokens = self.issue_and_store_tokens(&user.id, &user.role).await?;
        Ok(AuthResponse { user: UserProfile::from(user), tokens })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, DomainError> {
        let email = req.email.trim().to_lowercase();

        let user = self
            .repo
            .find_by_email(&email)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        if !user.is_active {
            return Err(DomainError::AccountInactive);
        }

        crypto::verify(&req.password, &user.password_hash)?;

        let tokens = self.issue_and_store_tokens(&user.id, &user.role).await?;
        Ok(AuthResponse { user: UserProfile::from(user), tokens })
    }

    pub async fn refresh_tokens(&self, req: RefreshTokenRequest) -> Result<AuthTokens, DomainError> {
        let claims = jwt::decode_token(&req.refresh_token, &self.config.jwt_refresh_secret)
            .map_err(|_| DomainError::InvalidToken)?;

        let user = self
            .repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or(DomainError::InvalidToken)?;

        if !user.is_active {
            return Err(DomainError::AccountInactive);
        }

        let stored_hash = user.refresh_token_hash.as_deref().ok_or(DomainError::InvalidToken)?;
        crypto::verify(&req.refresh_token, stored_hash)?;

        self.issue_and_store_tokens(&user.id, &user.role).await
    }

    pub async fn logout(&self, user_id: &str) -> Result<(), DomainError> {
        self.repo.update_refresh_token(user_id, None).await
    }

    pub async fn get_me(&self, user_id: &str) -> Result<UserProfile, DomainError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::UserNotFound)?;

        if !user.is_active {
            return Err(DomainError::AccountInactive);
        }
        Ok(UserProfile::from(user))
    }

    pub async fn update_me(
        &self,
        user_id: &str,
        req: UpdateProfileRequest,
    ) -> Result<UserProfile, DomainError> {
        let update = ProfileUpdate {
            full_name: req.full_name.map(|n| n.trim().to_string()),
            preferred_language: req.preferred_language,
            farm_info: req.farm_info.map(FarmInfo::from),
            preferences: req.preferences.map(UserPreferences::from),
        };
        let user = self.repo.update_profile(user_id, &update).await?;
        Ok(UserProfile::from(user))
    }

    async fn issue_and_store_tokens(
        &self,
        user_id: &str,
        role: &crate::domain::user::UserRole,
    ) -> Result<AuthTokens, DomainError> {
        let role_str = format!("{role:?}");

        let access_token = jwt::issue(user_id, &role_str, &self.config.jwt_access_secret, self.config.jwt_access_expiry_secs)
            .map_err(|e| DomainError::Internal(format!("Access token signing failed: {e}")))?;

        let raw_refresh = jwt::issue(user_id, &role_str, &self.config.jwt_refresh_secret, self.config.jwt_refresh_expiry_secs)
            .map_err(|e| DomainError::Internal(format!("Refresh token signing failed: {e}")))?;

        let refresh_hash = crypto::hash(&raw_refresh)?;
        self.repo.update_refresh_token(user_id, Some(&refresh_hash)).await?;

        Ok(AuthTokens {
            access_token,
            refresh_token: raw_refresh,
            token_type: "Bearer",
            expires_in: self.config.jwt_access_expiry_secs,
        })
    }
}

fn normalise_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    match digits.len() {
        12 if digits.starts_with("91") => digits[2..].to_string(),
        11 if digits.starts_with('0') => digits[1..].to_string(),
        _ => digits,
    }
}
