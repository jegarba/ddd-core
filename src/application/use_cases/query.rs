use std::marker::PhantomData;
use std::sync::Arc;

use crate::application::ports::ReadRepository;
use crate::domain::{AggregateRoot, DomainError};

pub struct QueryUseCase<T: AggregateRoot + Send + Sync, R: ReadRepository<T>> {
    repository: Arc<R>,
    _marker: PhantomData<T>,
}

impl<T: AggregateRoot + Send + Sync, R: ReadRepository<T>> QueryUseCase<T, R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository, _marker: PhantomData }
    }

    pub async fn find_by_id(&self, id: T::Id) -> Result<T, DomainError> {
        self.repository
            .find_by_id(&id)
            .await?
            .ok_or(DomainError::NotFound { entity: std::any::type_name::<T>() })
    }

    pub async fn list(&self, page: i64, limit: i64) -> Result<Vec<T>, DomainError> {
        let page = page.max(1);
        self.repository.find_all_paginated(limit, (page - 1) * limit).await
    }
}
