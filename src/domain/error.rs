#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("{entity} no encontrado")]
    NotFound { entity: &'static str },
    #[error("{entity} ya existe: {reason}")]
    AlreadyExists { entity: &'static str, reason: String },
    #[error("Regla de negocio violada: {0}")]
    InvariantViolation(String),
    #[error("Error de validación: {0}")]
    Validation(String),
    #[error("Error de infraestructura: {0}")]
    Infrastructure(String),
}
