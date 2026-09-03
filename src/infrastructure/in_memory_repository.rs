// ⚠️ SOLO PARA TESTS. Usa `std::sync::RwLock` (bloqueante) dentro de
// métodos `async` — un adapter real usa `tokio::sync::RwLock`, nunca esto.
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::application::ports::{ReadRepository, WriteRepository};
use crate::domain::{AggregateRoot, DomainError};

pub struct InMemoryRepository<T: AggregateRoot + Clone> {
    store: RwLock<HashMap<T::Id, T>>,
}

impl<T: AggregateRoot + Clone> Default for InMemoryRepository<T>
where
    T::Id: Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AggregateRoot + Clone> InMemoryRepository<T>
where
    T::Id: Hash,
{
    pub fn new() -> Self {
        Self { store: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl<T: AggregateRoot + Clone + Send + Sync> ReadRepository<T> for InMemoryRepository<T>
where
    T::Id: Hash,
{
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, DomainError> {
        Ok(self.store.read().unwrap().get(id).cloned())
    }
    async fn find_all_paginated(&self, limit: i64, offset: i64) -> Result<Vec<T>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl<T: AggregateRoot + Clone + Send + Sync> WriteRepository<T> for InMemoryRepository<T>
where
    T::Id: Hash,
{
    async fn create(&self, entity: T) -> Result<T, DomainError> {
        // Mismo contrato que Postgres: create() no exige id previo — el
        // dominio lo asigna en su propio CreationPolicy::build (ej. UUID).
        let id = entity.id().cloned().ok_or_else(|| {
            DomainError::Infrastructure(
                "entidad sin id en create() — el dominio debe asignarlo antes".into(),
            )
        })?;
        self.store.write().unwrap().insert(id, entity.clone());
        Ok(entity)
    }
    async fn update(&self, entity: T) -> Result<T, DomainError> {
        let id = entity
            .id()
            .cloned()
            .ok_or_else(|| DomainError::Infrastructure("entidad sin id en update()".into()))?;
        self.store.write().unwrap().insert(id, entity.clone());
        Ok(entity)
    }
    async fn delete(&self, id: &T::Id) -> Result<(), DomainError> {
        self.store.write().unwrap().remove(id);
        Ok(())
    }
}
