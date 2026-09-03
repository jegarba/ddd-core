/// OPTIONAL capability — a domain implements this only if its entities need
/// filtering by "something" (tenant, organization, workspace...). The kernel
/// doesn't define what a scope is, only reserves the shape.
pub trait Scoped {
    type ScopeId: Clone + Eq + Send + Sync;
    fn scope_id(&self) -> &Self::ScopeId;
}
