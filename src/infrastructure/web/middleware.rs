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

/// `tower_http` ships no request-id generator — this is the one the kernel
/// uses, UUID v4 per request.
#[derive(Clone, Default)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        axum::http::HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}

/// Applies the standard middleware stack (mold §1.3) to a Router. Fixed
/// order: panic-safety → request-id → trace → timeout → body-limit →
/// compression → CORS. Auth/rate-limit are optional, added separately
/// (see `application::ports::auth`).
pub fn apply_standard_middleware(router: axum::Router) -> axum::Router {
    let request_id_header = HeaderName::from_static(REQUEST_ID_HEADER);

    router.layer(
        ServiceBuilder::new()
            .layer(CatchPanicLayer::new())
            .layer(SetRequestIdLayer::new(request_id_header.clone(), MakeRequestUuid))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::new(request_id_header))
            .layer(TimeoutLayer::new(DEFAULT_TIMEOUT))
            // `DefaultBodyLimit` (axum), not `tower_http::limit::RequestBodyLimitLayer` —
            // the latter doesn't compose with CompressionLayer (the resulting
            // response body doesn't implement `Default`, real compile error hit).
            .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT_BYTES))
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive()),
    )
}
