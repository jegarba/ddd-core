use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;

/// Cualquier dependencia externa (pool de DB, cache) implementa esto — el
/// molde agrega los checks sin saber qué hay adentro de cada uno.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &'static str;
    async fn check(&self) -> Result<(), String>;
}

#[derive(Clone, Default)]
pub struct HealthRegistry {
    checks: Vec<Arc<dyn HealthCheck>>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, check: Arc<dyn HealthCheck>) -> Self {
        self.checks.push(check);
        self
    }

    /// GET /health → liveness, 200 si el proceso responde, sin chequear dependencias.
    pub async fn liveness() -> StatusCode {
        StatusCode::OK
    }

    /// GET /ready → readiness, corre los checks; 503 si alguno falla.
    pub async fn readiness(&self) -> StatusCode {
        for check in &self.checks {
            if check.check().await.is_err() {
                tracing::warn!(check = check.name(), "health check falló");
                return StatusCode::SERVICE_UNAVAILABLE;
            }
        }
        StatusCode::OK
    }
}
