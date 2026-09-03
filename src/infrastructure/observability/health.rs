use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;

/// Any external dependency (DB pool, cache) implements this — the kernel
/// aggregates checks without knowing what's inside each one.
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

    /// GET /health → liveness, 200 if the process responds, no dependency checks.
    pub async fn liveness() -> StatusCode {
        StatusCode::OK
    }

    /// GET /ready → readiness, runs the checks; 503 if any fails.
    pub async fn readiness(&self) -> StatusCode {
        for check in &self.checks {
            if let Err(reason) = check.check().await {
                tracing::warn!(check = check.name(), reason = %reason, "health check failed");
                return StatusCode::SERVICE_UNAVAILABLE;
            }
        }
        StatusCode::OK
    }
}
