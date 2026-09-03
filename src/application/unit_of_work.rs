/// Cuando un caso de uso necesita que dos escrituras de agregados distintos
/// sean atómicas, no se resuelve pasando `sqlx::Transaction` hacia
/// `application` (rompe Low Coupling) — se resuelve con un puerto que
/// exponga "ejecutá esto atómicamente" sin que el dominio sepa que por
/// debajo hay una transacción real.
///
/// Firma exacta: deliberadamente no cerrada — se diseña contra el primer
/// caso real con dos agregados en la misma transacción, no contra uno
/// hipotético.
pub trait UnitOfWork: Send + Sync {}
