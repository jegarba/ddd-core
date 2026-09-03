use async_trait::async_trait;

use crate::domain::{AggregateRoot, DomainError};

pub mod auth;

#[async_trait]
pub trait ReadRepository<T: AggregateRoot + Send + Sync>: Send + Sync {
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, DomainError>;
    async fn find_all_paginated(&self, limit: i64, offset: i64) -> Result<Vec<T>, DomainError>;
}

#[async_trait]
pub trait WriteRepository<T: AggregateRoot + Send + Sync>: Send + Sync {
    /// Recibe la entidad SIN id — la implementación lo asigna y lo devuelve
    /// en el resultado.
    async fn create(&self, entity: T) -> Result<T, DomainError>;
    /// Recibe la entidad CON id ya asignado — persiste el estado actual.
    async fn update(&self, entity: T) -> Result<T, DomainError>;
    async fn delete(&self, id: &T::Id) -> Result<(), DomainError>;
}
