use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Invalid specification format: {0}")]
    InvalidSpec(String),

    #[error("Snapshot error: {0}")]
    Snapshot(String),

    #[error("Diff error: {0}")]
    Diff(String),
}

pub type Result<T> = std::result::Result<T, ContractError>;
