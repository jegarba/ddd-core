use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// JSON in production (parseable by the observability stack), readable
/// locally (`debug_assertions`). Call once in `main()`.
pub fn init_tracing(_service_name: &'static str) {
    let registry = tracing_subscriber::registry().with(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        registry.with(tracing_subscriber::fmt::layer().pretty()).init();
    } else {
        registry.with(tracing_subscriber::fmt::layer().json()).init();
    }
}
