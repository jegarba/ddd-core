use tokio::sync::broadcast;

/// Cualquier evento de dominio solo necesita ser clonable y enviable entre
/// tareas — el molde no sabe qué pasó, solo transporta.
pub trait DomainEvent: Clone + Send + Sync + 'static {}

/// ⚠️ Best-effort, no confiable: `publish` descarta el resultado a propósito
/// — sin suscriptores, o con uno lento (`RecvError::Lagged`), el evento se
/// pierde en silencio. Para un efecto que NO puede perderse, usar el patrón
/// Outbox (tabla `outbox` en la misma transacción del agregado) — no
/// implementado acá, se construye contra el primer caso real.
pub struct EventBus<E: DomainEvent> {
    sender: broadcast::Sender<E>,
}

impl<E: DomainEvent> EventBus<E> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    pub fn publish(&self, event: E) {
        let _ = self.sender.send(event);
    }
    pub fn subscribe(&self) -> broadcast::Receiver<E> {
        self.sender.subscribe()
    }
}
