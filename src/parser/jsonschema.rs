use crate::error::Result;
use crate::model::*;
use crate::parser::openapi::parse_property;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_json_schema(val: &Value, format_source: &str) -> Result<CanonicalContract> {
    let title = val
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("RootModel");

    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut contract = CanonicalContract::new(title, format_source);
    contract.description = description;

    // 1. Root Entity
    let root_entity = parse_single_schema_entity(title, val);
    contract.add_entity(root_entity);

    // 2. Definitions / $defs
    if let Some(defs) = val
        .get("$defs")
        .or_else(|| val.get("definitions"))
        .and_then(|v| v.as_object())
    {
        for (def_name, def_val) in defs {
            let entity = parse_single_schema_entity(def_name, def_val);
            contract.add_entity(entity);
        }
    }

    Ok(contract)
}

fn parse_single_schema_entity(name: &str, schema: &Value) -> EntityContract {
    let description = schema.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let mut properties = BTreeMap::new();
    let mut required_properties = BTreeSet::new();

    if let Some(req_arr) = schema.get("required").and_then(|v| v.as_array()) {
        for r in req_arr {
            if let Some(field_name) = r.as_str() {
                required_properties.insert(field_name.to_string());
            }
        }
    }

    if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
        for (prop_name, prop_val) in props {
            let prop = parse_property(prop_name, prop_val);
            properties.insert(prop_name.to_string(), prop);
        }
    }

    let enum_variants = schema.get("enum").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|item| match item {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                _ => None,
            })
            .collect()
    });

    let additional_properties = schema
        .get("additionalProperties")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    EntityContract {
        name: name.to_string(),
        description,
        properties,
        required_properties,
        enum_variants,
        additional_properties,
    }
}
