use std::str::FromStr;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::application::ports::{ReadRepository, WriteRepository};
use crate::application::use_cases::{CreateUseCase, CreationPolicy, QueryUseCase};
use crate::domain::{AggregateRoot, DomainError};

#[derive(Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}

/// Wires `QueryUseCase`/`CreateUseCase` into standard CRUD routes.
/// Per-domain boilerplate: zero new handlers — just `RestResource::new(...).router()`.
///
/// Keeps the repository alongside the use cases because `delete` is a
/// plain write operation (no policy/business rule like `create` has) — not
/// worth a separate generic use case, it calls the write port directly.
pub struct RestResource<T, R, P>
where
    T: AggregateRoot + Send + Sync,
    R: ReadRepository<T> + WriteRepository<T>,
    P: CreationPolicy<T, T>,
{
    query: Arc<QueryUseCase<T, R>>,
    create: Arc<CreateUseCase<T, R, P>>,
    repository: Arc<R>,
}

impl<T, R, P> RestResource<T, R, P>
where
    T: AggregateRoot + Serialize + DeserializeOwned + Send + Sync + 'static,
    T::Id: FromStr + ToString + Send + Sync + 'static,
    R: ReadRepository<T> + WriteRepository<T> + 'static,
    P: CreationPolicy<T, T> + 'static,
{
    pub fn new(repository: Arc<R>, policy: Arc<P>) -> Self {
        Self {
            query: Arc::new(QueryUseCase::new(repository.clone())),
            create: Arc::new(CreateUseCase::new(repository.clone(), policy)),
            repository,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::list).post(Self::create_h))
            // axum 0.7.9 resolves to matchit 0.7.x — path param syntax is
            // `:id`, NOT `{id}` (that's axum 0.8/matchit 0.8). Real bug
            // found 2026-09-02 when the first test that fires a real HTTP
            // request against this route was added — no prior test did,
            // they only checked the Router compiled.
            .route("/:id", get(Self::get_h).put(Self::update_h).delete(Self::delete_h))
            .with_state(Arc::new(self))
    }

    async fn list(
        State(s): State<Arc<Self>>,
        Query(p): Query<PageParams>,
    ) -> Result<Json<Vec<T>>, DomainError> {
        Ok(Json(s.query.list(p.page, p.limit).await?))
    }

    async fn get_h(
        State(s): State<Arc<Self>>,
        Path(id): Path<String>,
    ) -> Result<Json<T>, DomainError> {
        let id = parse_id::<T>(&id)?;
        Ok(Json(s.query.find_by_id(id).await?))
    }

    async fn create_h(
        State(s): State<Arc<Self>>,
        Json(input): Json<T>,
    ) -> Result<(StatusCode, Json<T>), DomainError> {
        Ok((StatusCode::CREATED, Json(s.create.execute(input).await?)))
    }

    /// No generic `UpdateUseCase` — unlike `create` (which needs
    /// `CreationPolicy` to assign the id and validate creation invariants),
    /// `update` receives an already-complete entity: it calls the write
    /// port directly, same as `delete_h` already does. The URL id is the
    /// source of truth — if the body carries a different one (or none), it's
    /// rejected instead of letting the body silently override it.
    async fn update_h(
        State(s): State<Arc<Self>>,
        Path(id): Path<String>,
        Json(entity): Json<T>,
    ) -> Result<Json<T>, DomainError> {
        let path_id = parse_id::<T>(&id)?;
        match entity.id() {
            Some(body_id) if *body_id == path_id => {}
            Some(_) => {
                return Err(DomainError::Validation("body id does not match the URL id".into()))
            }
            None => {
                return Err(DomainError::Validation("body must include the id when updating".into()))
            }
        }
        Ok(Json(s.repository.update(entity).await?))
    }

    async fn delete_h(
        State(s): State<Arc<Self>>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, DomainError> {
        let id = parse_id::<T>(&id)?;
        s.repository.delete(&id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

fn parse_id<T: AggregateRoot>(raw: &str) -> Result<T::Id, DomainError>
where
    T::Id: FromStr,
{
    raw.parse::<T::Id>().map_err(|_| DomainError::Validation(format!("invalid id: {raw}")))
}
