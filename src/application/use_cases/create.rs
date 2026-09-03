use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::application::ports::WriteRepository;
use crate::domain::{AggregateRoot, DomainError};

/// Each domain implements this ONCE — the domain-specific rule (e.g. "email
/// must be unique") lives here, not in the kernel.
#[async_trait]
pub trait CreationPolicy<T: AggregateRoot, Input>: Send + Sync {
    async fn build(&self, input: Input) -> Result<T, DomainError>;
    /// GLOBAL invariant check that needs the repository (e.g. uniqueness) —
    /// the kernel orchestrates when it's called, the domain decides what to check.
    async fn check_invariants(&self, entity: &T) -> Result<(), DomainError>;
}

/// Does not publish events itself — if the creation event matters, the
/// caller (REST handler, or another orchestrator) publishes it after
/// `execute()` with its own `EventBus`. Holding an unused `EventBus` here
/// would be dead code — YAGNI.
pub struct CreateUseCase<T, R, P>
where
    T: AggregateRoot + Send + Sync,
    R: WriteRepository<T>,
    P: CreationPolicy<T, T>,
{
    repository: Arc<R>,
    policy: Arc<P>,
    _marker: PhantomData<T>,
}

impl<T, R, P> CreateUseCase<T, R, P>
where
    T: AggregateRoot + Send + Sync,
    R: WriteRepository<T>,
    P: CreationPolicy<T, T>,
{
    pub fn new(repository: Arc<R>, policy: Arc<P>) -> Self {
        Self { repository, policy, _marker: PhantomData }
    }

    pub async fn execute(&self, input: T) -> Result<T, DomainError> {
        let entity = self.policy.build(input).await?;
        self.policy.check_invariants(&entity).await?;
        self.repository.create(entity).await
    }
}
