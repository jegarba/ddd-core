use async_trait::async_trait;

use crate::domain::DomainError;

/// Result of authenticating a request — opaque: doesn't care whether the
/// credential was a JWT, an API key, or anything else.
pub trait Principal: Send + Sync {
    fn subject_id(&self) -> &str;
    fn has_permission(&self, permission: &str) -> bool;
}

/// The concrete project implements this against whatever it uses (or
/// doesn't, if the service needs no auth). The kernel never imports a provider.
#[async_trait]
pub trait Authenticator: Send + Sync {
    type P: Principal;
    async fn authenticate(&self, raw_credential: &str) -> Result<Self::P, DomainError>;
}
