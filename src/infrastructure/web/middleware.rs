use std::time::Duration;

use axum::{extract::{DefaultBodyLimit, Request}, http::HeaderName};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";
const DEFAULT_BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// `tower_http` no trae un generador de request-id incluido — este es el
/// que usa el molde, UUID v4 por request.
#[derive(Clone, Default)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        axum::http::HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}

/// Aplica el stack estándar de middleware (§1.3 del molde) a un Router.
/// Orden fijo: panic-safety → request-id → trace → timeout → body-limit →
/// compression → CORS. Auth/rate-limit son opcionales, se agregan aparte
/// (ver `application::ports::auth`).
pub fn apply_standard_middleware(router: axum::Router) -> axum::Router {
    let request_id_header = HeaderName::from_static(REQUEST_ID_HEADER);

    router.layer(
        ServiceBuilder::new()
            .layer(CatchPanicLayer::new())
            .layer(SetRequestIdLayer::new(request_id_header.clone(), MakeRequestUuid))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::new(request_id_header))
            .layer(TimeoutLayer::new(DEFAULT_TIMEOUT))
            // `DefaultBodyLimit` (axum), no `tower_http::limit::RequestBodyLimitLayer` —
            // ese último no compone con CompressionLayer (el body de respuesta
            // resultante no implementa `Default`, error real encontrado al compilar).
            .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT_BYTES))
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive()),
    )
}
