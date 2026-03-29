use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,

    #[error("Email address is already in use")]
    EmailConflict,

    #[error("Phone number is already in use")]
    PhoneConflict,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Account is inactive or has been suspended")]
    AccountInactive,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
