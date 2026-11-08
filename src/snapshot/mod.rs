use crate::error::{ContractError, Result};
use crate::model::CanonicalContract;
use crate::parser::parse_spec_file;
use std::fs;
use std::path::Path;

pub fn create_snapshot<P: AsRef<Path>>(spec_path: P) -> Result<CanonicalContract> {
    parse_spec_file(spec_path)
}

pub fn save_snapshot<P: AsRef<Path>>(contract: &CanonicalContract, out_path: P) -> Result<()> {
    let path = out_path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json_str = serde_json::to_string_pretty(contract)
        .map_err(|e| ContractError::Snapshot(format!("Failed to serialize contract: {e}")))?;
    fs::write(path, json_str)?;
    Ok(())
}

pub fn load_snapshot<P: AsRef<Path>>(baseline_path: P) -> Result<CanonicalContract> {
    let path = baseline_path.as_ref();
    if !path.exists() {
        return Err(ContractError::Snapshot(format!(
            "Baseline snapshot not found at: {}",
            path.display()
        )));
    }
    let content = fs::read_to_string(path)?;
    // If it's a raw canonical snapshot JSON
    if let Ok(contract) = serde_json::from_str::<CanonicalContract>(&content) {
        return Ok(contract);
    }
    // Alternatively if the baseline provided is another spec file (OpenAPI/JSON Schema)
    parse_spec_file(path)
}
