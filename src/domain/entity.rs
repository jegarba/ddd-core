/// Anything with identity. On its own does NOT imply its own repository —
/// that's decided by `AggregateRoot`.
pub trait Entity {
    type Id: Clone + Eq + Send + Sync;
    fn id(&self) -> Option<&Self::Id>;
}

/// Marker trait: only aggregate root entities get a repository. A child
/// entity (e.g. a line item inside an order) implements `Entity` but NEVER
/// `AggregateRoot` — it's persisted through its root. This avoids the most
/// common mistake when "genericizing" DDD: giving every table its own
/// repository and losing the invariants the aggregate exists to protect.
pub trait AggregateRoot: Entity {}
