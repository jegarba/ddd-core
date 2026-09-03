use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::application::ports::WriteRepository;
use crate::domain::{AggregateRoot, DomainError};

/// Cada dominio implementa esto UNA vez — acá vive la regla específica
/// (ej: "el email no puede repetirse"), no en el kernel.
#[async_trait]
pub trait CreationPolicy<T: AggregateRoot, Input>: Send + Sync {
    async fn build(&self, input: Input) -> Result<T, DomainError>;
    /// Chequeo de invariante GLOBAL que necesita consultar el repositorio
    /// (ej. unicidad) — el kernel orquesta cuándo se llama, el dominio
    /// decide qué chequear.
    async fn check_invariants(&self, entity: &T) -> Result<(), DomainError>;
}

/// No publica eventos por sí mismo — si el evento de creación importa,
/// el caller (el handler REST, u otro orquestador) lo publica después de
/// `execute()` con su propio `EventBus`. Cargar un `EventBus` acá adentro
/// sin usarlo sería dead code — YAGNI.
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
