use tokio::sync::broadcast;

/// Any domain event just needs to be cloneable and sendable across tasks —
/// the kernel doesn't know what happened, it only transports.
pub trait DomainEvent: Clone + Send + Sync + 'static {}

/// Best-effort, not reliable: `publish` discards the result on purpose — with
/// no subscribers, or a slow one (`RecvError::Lagged`), the event is silently
/// dropped. For an effect that CANNOT be lost, use the Outbox pattern (an
/// `outbox` table in the same transaction as the aggregate) — not
/// implemented here, build it against the first real case that needs it.
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
