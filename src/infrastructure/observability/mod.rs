pub mod health;
pub mod logging;
pub mod shutdown;

pub use health::{HealthCheck, HealthRegistry};
pub use logging::init_tracing;
pub use shutdown::signal;

// Métricas (Prometheus/`metrics` crate): despriorizado a pedido — es "un
// adapter más" (§2.3 del mold), no bloquea el arranque de los primeros
// microservicios. Se agrega cuando haga falta medir algo puntual.
