pub mod health;
pub mod logging;
pub mod shutdown;

pub use health::{HealthCheck, HealthRegistry};
pub use logging::init_tracing;
pub use shutdown::signal;

// Metrics (Prometheus/`metrics` crate): deprioritized on request — it's
// "one more adapter" (mold §2.3), doesn't block the first microservices
// from shipping. Add it when there's something concrete to measure.
