use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::application::ports::ReadRepository;
use crate::domain::{AggregateRoot, DomainError};

/// Contrato mínimo — el molde no elige Redis, LRU en memoria, ni nada concreto.
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V, ttl: Duration);
    async fn invalidate(&self, key: &K);
}

/// Decorator sobre `ReadRepository<T>` — `application` nunca se entera de
/// que existe, sigue usando el mismo `QueryUseCase` de siempre.
pub struct CachedReadRepository<T, R, C> {
    inner: Arc<R>,
    cache: Arc<C>,
    ttl: Duration,
    _marker: PhantomData<T>,
}

impl<T, R, C> CachedReadRepository<T, R, C> {
    pub fn new(inner: Arc<R>, cache: Arc<C>, ttl: Duration) -> Self {
        Self { inner, cache, ttl, _marker: PhantomData }
    }
}

#[async_trait]
impl<T, R, C> ReadRepository<T> for CachedReadRepository<T, R, C>
where
    T: AggregateRoot + Clone + Send + Sync,
    R: ReadRepository<T>,
    C: Cache<T::Id, T>,
{
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, DomainError> {
        if let Some(hit) = self.cache.get(id).await {
            return Ok(Some(hit));
        }
        let result = self.inner.find_by_id(id).await?;
        if let Some(v) = &result {
            self.cache.set(id.clone(), v.clone(), self.ttl).await;
        }
        Ok(result)
    }
    async fn find_all_paginated(&self, limit: i64, offset: i64) -> Result<Vec<T>, DomainError> {
        // Listados no cachean 1:1 por defecto — cada proyecto decide si vale la pena acá.
        self.inner.find_all_paginated(limit, offset).await
    }
}
