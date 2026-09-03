use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::application::ports::ReadRepository;
use crate::domain::{AggregateRoot, DomainError};

/// Minimal contract — the kernel doesn't pick Redis, in-memory LRU, or
/// anything concrete.
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V, ttl: Duration);
    async fn invalidate(&self, key: &K);
}

/// Decorator over `ReadRepository<T>` — `application` never finds out it
/// exists, it keeps using the same `QueryUseCase` as always.
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
        // Listings aren't cached 1:1 by default — each project decides if it's worth it here.
        self.inner.find_all_paginated(limit, offset).await
    }
}
