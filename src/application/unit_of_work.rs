/// When a use case needs two different aggregates' writes to be atomic, it's
/// not solved by passing `sqlx::Transaction` into `application` (breaks Low
/// Coupling) — it's solved with a port that exposes "run this atomically"
/// without the domain knowing there's a real transaction underneath.
///
/// Exact signature deliberately left open — designed against the first real
/// case with two aggregates in one transaction, not a hypothetical one.
pub trait UnitOfWork: Send + Sync {}
