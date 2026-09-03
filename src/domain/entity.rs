/// Cualquier cosa con identidad. Por sí sola NO implica repositorio propio
/// — eso lo decide `AggregateRoot`.
pub trait Entity {
    type Id: Clone + Eq + Send + Sync;
    fn id(&self) -> Option<&Self::Id>;
}

/// Marker trait: solo las Entidades raíz de un agregado tienen repositorio.
/// Una entidad hija (ej. una línea dentro de un pedido) implementa `Entity`
/// pero NUNCA `AggregateRoot` — se persiste a través de su root. Esto evita
/// el error más común al "genericizar" DDD: darle repositorio a cada tabla
/// y perder los invariantes que el agregado existe para proteger.
pub trait AggregateRoot: Entity {}
