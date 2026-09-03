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
    /// Receives the entity WITHOUT an id — the implementation assigns one
    /// and returns it in the result.
    async fn create(&self, entity: T) -> Result<T, DomainError>;
    /// Receives the entity WITH an id already assigned — persists current state.
    async fn update(&self, entity: T) -> Result<T, DomainError>;
    async fn delete(&self, id: &T::Id) -> Result<(), DomainError>;
}
