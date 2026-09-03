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

/// Conecta `QueryUseCase`/`CreateUseCase` a rutas CRUD estándar. Boilerplate
/// por dominio: cero handlers nuevos — solo `RestResource::new(...).router()`.
///
/// Guarda el repositorio además de los use cases porque `delete` es una
/// operación de escritura simple (sin política/regla de negocio propia como
/// `create`) — no amerita un caso de uso genérico aparte, se llama directo
/// al puerto de escritura.
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
            // axum 0.7.9 resuelve matchit 0.7.x — sintaxis de path param es
            // `:id`, NO `{id}` (eso es de axum 0.8/matchit 0.8). Bug real
            // encontrado 2026-09-02 al agregar el primer test que dispara un
            // request HTTP de verdad contra esta ruta — ningún test previo
            // lo hacía, solo verificaban que el Router compilara.
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

    /// No hay `UpdateUseCase` genérico — a diferencia de `create` (que
    /// necesita `CreationPolicy` para asignar el id y validar invariantes de
    /// creación), `update` recibe la entidad ya completa y madura: se llama
    /// directo al puerto de escritura, mismo criterio que `delete_h` ya usa.
    /// El id de la URL es la fuente de verdad — si el body trae uno distinto
    /// (o ninguno), se rechaza en vez de dejar que el body lo pise en
    /// silencio.
    async fn update_h(
        State(s): State<Arc<Self>>,
        Path(id): Path<String>,
        Json(entity): Json<T>,
    ) -> Result<Json<T>, DomainError> {
        let path_id = parse_id::<T>(&id)?;
        match entity.id() {
            Some(body_id) if *body_id == path_id => {}
            Some(_) => {
                return Err(DomainError::Validation(
                    "el id del body no coincide con el id de la URL".into(),
                ))
            }
            None => {
                return Err(DomainError::Validation(
                    "el body debe incluir el id al actualizar".into(),
                ))
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
    raw.parse::<T::Id>()
        .map_err(|_| DomainError::Validation(format!("id inválido: {raw}")))
}
