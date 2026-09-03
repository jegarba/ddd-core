// RFC 7807 (Problem Details for HTTP APIs) — adoptado del proyecto
// `contract_parent` (módulo `party`), que ya lo tenía probado en producción.
// Más estándar que un envelope de error propio inventado.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;

use crate::domain::DomainError;

const BASE_URI: &str = "https://api.ddd-core.internal/problems/";

#[derive(Serialize)]
pub struct ProblemDetail {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    /// El path del request — se completa vía el request-id middleware
    /// cuando corre dentro de un handler; `None` fuera de ese contexto.
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, String>>,
}

impl DomainError {
    fn problem_type_and_status(&self) -> (&'static str, StatusCode) {
        match self {
            DomainError::NotFound { .. } => ("not-found", StatusCode::NOT_FOUND),
            DomainError::AlreadyExists { .. } => ("already-exists", StatusCode::CONFLICT),
            DomainError::InvariantViolation(_) => {
                ("invariant-violation", StatusCode::UNPROCESSABLE_ENTITY)
            }
            DomainError::Validation(_) => ("validation-failed", StatusCode::BAD_REQUEST),
            DomainError::Infrastructure(_) => {
                ("internal-server-error", StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let (problem_slug, status) = self.problem_type_and_status();
        let body = ProblemDetail {
            problem_type: format!("{BASE_URI}{problem_slug}"),
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: self.to_string(),
            instance: None,
            errors: None,
        };
        let mut response = (status, Json(body)).into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}
