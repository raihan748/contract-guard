//! contract-guard
//! API schema & function contract regression snapshotter and semantic breaking change differ.

pub mod diff;
pub mod error;
pub mod model;
pub mod parser;
pub mod snapshot;

pub use diff::{ChangeKind, ContractChange, DiffEngine, DiffResult, Severity};
pub use error::{ContractError, Result};
pub use model::{
    CanonicalContract, EndpointContract, EntityContract, FieldType, ParameterContract,
    ParameterLocation, PrimitiveType, PropertyContract, RequestBodyContract, ResponseContract,
};
pub use parser::{parse_spec, parse_spec_file};
pub use snapshot::{create_snapshot, load_snapshot, save_snapshot};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
