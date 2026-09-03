use async_trait::async_trait;

use crate::domain::DomainError;

/// Resultado de autenticar una petición — opaco: no le importa si la
/// credencial era un JWT, una API key, o lo que sea.
pub trait Principal: Send + Sync {
    fn subject_id(&self) -> &str;
    fn has_permission(&self, permission: &str) -> bool;
}

/// El proyecto concreto implementa esto contra lo que use (o no lo usa,
/// si el servicio no necesita auth). El molde nunca importa un proveedor.
#[async_trait]
pub trait Authenticator: Send + Sync {
    type P: Principal;
    async fn authenticate(&self, raw_credential: &str) -> Result<Self::P, DomainError>;
}
