pub mod jsonschema;
pub mod openapi;

use crate::error::{ContractError, Result};
use crate::model::CanonicalContract;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn parse_spec_file<P: AsRef<Path>>(path: P) -> Result<CanonicalContract> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(ContractError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Spec file not found: {}", path_ref.display()),
        )));
    }

    let content = fs::read_to_string(path_ref)?;
    parse_spec(&content)
}

pub fn parse_spec(content: &str) -> Result<CanonicalContract> {
    // 1. Parse into serde_json::Value (try JSON, fallback YAML)
    let val: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => serde_yaml::from_str(content).map_err(|e| {
            ContractError::InvalidSpec(format!("Failed to parse as JSON or YAML: {e}"))
        })?,
    };

    // 2. Identify spec format
    if let Some(openapi_ver) = val.get("openapi").and_then(|v| v.as_str()) {
        if openapi_ver.starts_with("3.1") {
            return openapi::parse_openapi(&val, "openapi_3.1");
        } else {
            return openapi::parse_openapi(&val, "openapi_3.0");
        }
    }

    if let Some(schema_url) = val.get("$schema").and_then(|v| v.as_str()) {
        if schema_url.contains("2020-12") {
            return jsonschema::parse_json_schema(&val, "json_schema_2020_12");
        } else if schema_url.contains("draft-07") {
            return jsonschema::parse_json_schema(&val, "json_schema_draft7");
        } else {
            return jsonschema::parse_json_schema(&val, "json_schema");
        }
    }

    // Heuristics
    if val.get("paths").is_some() || val.get("components").is_some() {
        return openapi::parse_openapi(&val, "openapi_3.0");
    }

    if val.get("properties").is_some() || val.get("definitions").is_some() || val.get("$defs").is_some() {
        return jsonschema::parse_json_schema(&val, "json_schema");
    }

    // Fallback: structured entity
    jsonschema::parse_json_schema(&val, "custom_schema")
}
