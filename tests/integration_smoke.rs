//! Prueba de humo: un dominio de juguete (`Widget`) atravesando TODO el
//! pipeline genérico — Entity/AggregateRoot, InMemoryRepository,
//! QueryUseCase, CreateUseCase+CreationPolicy, y RestResource (arma el
//! Router real). Si esto compila y pasa, el molde funciona de punta a punta,
//! no solo en el papel.
use std::sync::Arc;

use async_trait::async_trait;
use ddd_core::application::{CreationPolicy, QueryUseCase, ReadRepository, WriteRepository};
use ddd_core::domain::{AggregateRoot, DomainError, Entity};
use ddd_core::infrastructure::InMemoryRepository;
use ddd_core::infrastructure::web::RestResource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Widget {
    id: Option<Uuid>,
    name: String,
}

impl Entity for Widget {
    type Id = Uuid;
    fn id(&self) -> Option<&Self::Id> {
        self.id.as_ref()
    }
}
impl AggregateRoot for Widget {}

struct UniqueNamePolicy;

#[async_trait]
impl CreationPolicy<Widget, Widget> for UniqueNamePolicy {
    async fn build(&self, input: Widget) -> Result<Widget, DomainError> {
        Ok(input)
    }
    async fn check_invariants(&self, entity: &Widget) -> Result<(), DomainError> {
        if entity.name.trim().is_empty() {
            return Err(DomainError::Validation("name no puede estar vacío".into()));
        }
        Ok(())
    }
}

#[tokio::test]
async fn create_use_case_persists_via_in_memory_repository() {
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    let policy = Arc::new(UniqueNamePolicy);
    let create = ddd_core::application::CreateUseCase::new(repo.clone(), policy);

    let widget = Widget { id: Some(Uuid::new_v4()), name: "tornillo".into() };
    let created = create.execute(widget.clone()).await.expect("create debe funcionar");
    assert_eq!(created, widget);

    let query = QueryUseCase::new(repo);
    let found = query.find_by_id(widget.id.unwrap()).await.expect("debe encontrarlo");
    assert_eq!(found, widget);
}

#[tokio::test]
async fn create_use_case_rejects_invalid_invariant() {
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    let policy = Arc::new(UniqueNamePolicy);
    let create = ddd_core::application::CreateUseCase::new(repo, policy);

    let widget = Widget { id: Some(Uuid::new_v4()), name: "".into() };
    let err = create.execute(widget).await.expect_err("debe rechazar nombre vacío");
    assert!(matches!(err, DomainError::Validation(_)));
}

#[tokio::test]
async fn query_use_case_list_paginates() {
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    for i in 0..5 {
        repo.create(Widget { id: Some(Uuid::new_v4()), name: format!("w{i}") })
            .await
            .unwrap();
        // create() viene de WriteRepository — importarlo para poder llamarlo directo acá.
    }
    let query = QueryUseCase::new(repo);
    let page = query.list(1, 3).await.unwrap();
    assert_eq!(page.len(), 3);
}

#[test]
fn rest_resource_router_builds_for_a_concrete_domain() {
    // No levanta un server real — solo prueba que el tipado genérico de
    // RestResource<Widget, InMemoryRepository<Widget>, UniqueNamePolicy>
    // efectivamente compila y arma un Router válido para un dominio concreto.
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    let policy = Arc::new(UniqueNamePolicy);
    let resource = RestResource::new(repo, policy);
    let _router: axum::Router = resource.router();
}

// ── PUT /:id (update) — egyptians-ms-product FR-013 ─────────────────────────
// A diferencia de las pruebas de arriba (llaman los use cases directo), estas
// manejan requests HTTP reales contra el Router — es la única forma de probar
// que la ruta PUT quedó realmente cableada, no solo que el tipo compila.
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn body_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn update_route_persists_via_write_repository() {
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    let id = Uuid::new_v4();
    repo.create(Widget { id: Some(id), name: "original".into() }).await.unwrap();

    let router = RestResource::new(repo.clone(), Arc::new(UniqueNamePolicy)).router();

    let updated = Widget { id: Some(id), name: "actualizado".into() };
    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/{id}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&updated).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let got: Widget = body_json(res).await;
    assert_eq!(got.name, "actualizado");

    // el id no cambió — solo se actualizó el contenido
    let found = repo.find_by_id(&id).await.unwrap().unwrap();
    assert_eq!(found.name, "actualizado");
}

#[tokio::test]
async fn update_route_rejects_mismatched_body_id() {
    let repo = Arc::new(InMemoryRepository::<Widget>::new());
    let path_id = Uuid::new_v4();
    let body_id = Uuid::new_v4();
    repo.create(Widget { id: Some(path_id), name: "original".into() }).await.unwrap();

    let router = RestResource::new(repo, Arc::new(UniqueNamePolicy)).router();

    let mismatched = Widget { id: Some(body_id), name: "x".into() };
    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/{path_id}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&mismatched).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    // 400 exacto, no solo "algún 4xx" — un 404 también pasaría
    // is_client_error() y hubiera enmascarado el bug de sintaxis de ruta
    // que este mismo test ayudó a encontrar (ver rest_resource.rs).
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}


