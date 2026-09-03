use std::sync::Arc;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::{from_fn, Next},
    response::Response,
    Router,
};

use crate::application::ports::auth::Authenticator;

/// Envuelve un `Router` exigiendo autenticación en todas sus rutas, vía el
/// `Authenticator` que el proyecto concreto haya implementado (JWT, API
/// key, lo que sea — el molde no lo sabe). Inserta el `Principal`
/// resultante como request extension — los handlers lo leen con
/// `Extension<A::P>`.
///
/// No usa `State` del Router a propósito: así no colisiona con el estado
/// que `RestResource` ya usa para sus propios handlers — el
/// `authenticator` queda capturado en el closure, no en el estado del Router.
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

/// `Authorization: Bearer <token>` por defecto — un proyecto con otro
/// esquema de credencial (cookie, API key en header propio) reimplementa
/// esta función, el resto de `require_auth` no cambia.
fn extract_credential(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
}
