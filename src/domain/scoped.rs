/// Capacidad OPCIONAL — un dominio la implementa solo si sus Entidades
/// necesitan filtrarse por "algo" (tenant, organización, workspace...). El
/// molde no define qué es un scope; solo reserva la forma.
pub trait Scoped {
    type ScopeId: Clone + Eq + Send + Sync;
    fn scope_id(&self) -> &Self::ScopeId;
}
