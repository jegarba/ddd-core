#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("{entity} not found")]
    NotFound { entity: &'static str },
    #[error("{entity} already exists: {reason}")]
    AlreadyExists { entity: &'static str, reason: String },
    #[error("Business rule violated: {0}")]
    InvariantViolation(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
}
