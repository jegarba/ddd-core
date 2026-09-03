use std::sync::Arc;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::{from_fn, Next},
    response::Response,
    Router,
};

use crate::application::ports::auth::Authenticator;

/// Wraps a `Router`, requiring authentication on all its routes via
/// whatever `Authenticator` the concrete project implemented (JWT, API key,
/// anything — the kernel doesn't know). Inserts the resulting `Principal`
/// as a request extension — handlers read it with `Extension<A::P>`.
///
/// Deliberately doesn't use the Router's `State`: that way it never
/// collides with the state `RestResource` already uses for its own
/// handlers — `authenticator` stays captured in the closure, not in the
/// Router's state.
pub fn require_auth<A>(router: Router, authenticator: Arc<A>) -> Router
where
    A: Authenticator + Send + Sync + 'static,
    A::P: Clone + Send + Sync + 'static,
{
    router.layer(from_fn(move |req: Request, next: Next| {
        let authenticator = authenticator.clone();
        async move { auth_middleware(authenticator, req, next).await }
    }))
}

async fn auth_middleware<A>(
    authenticator: Arc<A>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode>
where
    A: Authenticator,
    A::P: Clone + Send + Sync + 'static,
{
    let raw = extract_credential(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    let principal = authenticator
        .authenticate(&raw)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}

/// `Authorization: Bearer <token>` by default — a project with a different
/// credential scheme (cookie, custom header API key) reimplements this
/// function, the rest of `require_auth` doesn't change.
fn extract_credential(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
}
