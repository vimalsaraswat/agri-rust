use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::domain::errors::DomainError;

pub fn hash(password: &str) -> Result<String, DomainError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| DomainError::Internal(format!("Password hashing failed: {e}")))
}

pub fn verify(candidate: &str, hash: &str) -> Result<(), DomainError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| DomainError::Internal("Malformed password hash".into()))?;
    Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed)
        .map_err(|_| DomainError::InvalidCredentials)
}
