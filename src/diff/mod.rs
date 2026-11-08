pub mod change;
pub mod engine;

pub use change::{ChangeKind, ContractChange, Severity};
pub use engine::{DiffEngine, DiffResult};
