use crate::error::Result;
use crate::model::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_openapi(val: &Value, format_source: &str) -> Result<CanonicalContract> {
    let title = val
        .pointer("/info/title")
        .and_then(|v| v.as_str())
        .unwrap_or("API Contract");

    let description = val
        .pointer("/info/description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut contract = CanonicalContract::new(title, format_source);
    contract.description = description;

    // 1. Extract Components / Schemas (or definitions)
    if let Some(schemas) = val
        .pointer("/components/schemas")
        .or_else(|| val.pointer("/definitions"))
        .and_then(|v| v.as_object())
    {
        for (name, schema_val) in schemas {
            let entity = parse_schema_entity(name, schema_val);
            contract.add_entity(entity);
        }
    }

    // 2. Extract Paths
    if let Some(paths) = val.pointer("/paths").and_then(|v| v.as_object()) {
        for (path_str, path_item) in paths {
            if !path_str.starts_with('/') {
                continue;
            }

            // Path-level parameters
            let path_params = extract_parameters(path_item.get("parameters"));

            // Methods: get, post, put, delete, patch, options, head
            for method in &["get", "post", "put", "delete", "patch", "options", "head"] {
                if let Some(op) = path_item.get(*method) {
                    let mut endpoint = parse_endpoint(method, path_str, op);
                    // Merge path-level parameters if not overridden
                    for p in &path_params {
                        if !endpoint.parameters.iter().any(|existing| existing.name == p.name && existing.location == p.location) {
                            endpoint.parameters.push(p.clone());
                        }
                    }
                    contract.add_endpoint(endpoint);
                }
            }
        }
    }

    Ok(contract)
}

fn parse_schema_entity(name: &str, schema: &Value) -> EntityContract {
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

    let enum_variants = extract_enums(schema.get("enum"));
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

pub fn parse_property(name: &str, val: &Value) -> PropertyContract {
    let (prop_type, nullable) = parse_field_type_and_nullability(val);
    let default_value = val.get("default").cloned();
    let enum_values = extract_enums(val.get("enum"));
    let format = val.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    PropertyContract {
        name: name.to_string(),
        prop_type,
        nullable,
        default_value,
        enum_values,
        format,
        description,
    }
}

pub fn parse_field_type_and_nullability(val: &Value) -> (FieldType, bool) {
    let mut nullable = val.get("nullable").and_then(|v| v.as_bool()).unwrap_or(false);

    // Reference
    if let Some(ref_str) = val.get("$ref").and_then(|v| v.as_str()) {
        let clean_ref = ref_str.rsplit('/').next().unwrap_or(ref_str);
        return (FieldType::Reference(clean_ref.to_string()), nullable);
    }

    // oneOf / anyOf unions
    if let Some(union_arr) = val.get("oneOf").or_else(|| val.get("anyOf")).and_then(|v| v.as_array()) {
        let mut types = Vec::new();
        for item in union_arr {
            let (ft, item_null) = parse_field_type_and_nullability(item);
            if item_null || ft == FieldType::Primitive(PrimitiveType::Null) {
                nullable = true;
            }
            if ft != FieldType::Primitive(PrimitiveType::Null) {
                types.push(ft);
            }
        }
        if types.len() == 1 {
            return (types.remove(0), nullable);
        } else if !types.is_empty() {
            return (FieldType::Union(types), nullable);
        }
    }

    // Type field
    if let Some(type_val) = val.get("type") {
        if let Some(type_str) = type_val.as_str() {
            let prim = PrimitiveType::from_str_loose(type_str);
            if prim == PrimitiveType::Null {
                return (FieldType::Primitive(PrimitiveType::Null), true);
            }
            if prim == PrimitiveType::Array {
                let inner_type = val
                    .get("items")
                    .map(|items_val| parse_field_type_and_nullability(items_val).0)
                    .unwrap_or(FieldType::Any);
                return (FieldType::Array(Box::new(inner_type)), nullable);
            }
            return (FieldType::Primitive(prim), nullable);
        } else if let Some(type_arr) = type_val.as_array() {
            // e.g. ["string", "null"] in JSON Schema / OpenAPI 3.1
            let mut prim_types = Vec::new();
            for t in type_arr {
                if let Some(s) = t.as_str() {
                    let prim = PrimitiveType::from_str_loose(s);
                    if prim == PrimitiveType::Null {
                        nullable = true;
                    } else {
                        prim_types.push(FieldType::Primitive(prim));
                    }
                }
            }
            if prim_types.len() == 1 {
                return (prim_types.remove(0), nullable);
            } else if !prim_types.is_empty() {
                return (FieldType::Union(prim_types), nullable);
            }
        }
    }

    // Object without explicit type
    if val.get("properties").is_some() {
        return (FieldType::Object, nullable);
    }

    (FieldType::Any, nullable)
}

fn extract_enums(val: Option<&Value>) -> Option<Vec<String>> {
    val.and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|item| match item {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                _ => None,
            })
            .collect()
    })
}

fn extract_parameters(val: Option<&Value>) -> Vec<ParameterContract> {
    let mut list = Vec::new();
    if let Some(arr) = val.and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(param) = parse_single_parameter(item) {
                list.push(param);
            }
        }
    }
    list
}

fn parse_single_parameter(val: &Value) -> Option<ParameterContract> {
    let name = val.get("name")?.as_str()?.to_string();
    let location_str = val.get("in").and_then(|v| v.as_str()).unwrap_or("query");
    let location = match location_str {
        "path" => ParameterLocation::Path,
        "query" => ParameterLocation::Query,
        "header" => ParameterLocation::Header,
        "cookie" => ParameterLocation::Cookie,
        _ => ParameterLocation::Query,
    };
    let required = val.get("required").and_then(|v| v.as_bool()).unwrap_or(location == ParameterLocation::Path);
    let schema_type = val
        .get("schema")
        .map(|s| parse_field_type_and_nullability(s).0)
        .unwrap_or(FieldType::Primitive(PrimitiveType::String));

    let default_value = val
        .get("schema")
        .and_then(|s| s.get("default"))
        .or_else(|| val.get("default"))
        .cloned();

    let description = val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    Some(ParameterContract {
        name,
        location,
        required,
        schema_type,
        default_value,
        description,
    })
}

fn parse_endpoint(method: &str, path: &str, op: &Value) -> EndpointContract {
    let summary = op.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
    let operation_id = op.get("operationId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let deprecated = op.get("deprecated").and_then(|v| v.as_bool()).unwrap_or(false);

    let parameters = extract_parameters(op.get("parameters"));

    let request_body = op.get("requestBody").map(|rb| {
        let required = rb.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
        let description = rb.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        let mut content_types = Vec::new();
        let mut schema_ref = None;
        let mut inline_schema = None;

        if let Some(content) = rb.get("content").and_then(|v| v.as_object()) {
            for (ct, media) in content {
                content_types.push(ct.clone());
                if let Some(schema) = media.get("schema") {
                    let (ft, _) = parse_field_type_and_nullability(schema);
                    match ft {
                        FieldType::Reference(r) => schema_ref = Some(r),
                        other => inline_schema = Some(other),
                    }
                }
            }
        }

        RequestBodyContract {
            required,
            content_types,
            schema_ref,
            inline_schema,
            description,
        }
    });

    let mut responses = BTreeMap::new();
    if let Some(resp_obj) = op.get("responses").and_then(|v| v.as_object()) {
        for (code, resp_val) in resp_obj {
            let description = resp_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let mut content_types = Vec::new();
            let mut schema_ref = None;
            let mut inline_schema = None;

            if let Some(content) = resp_val.get("content").and_then(|v| v.as_object()) {
                for (ct, media) in content {
                    content_types.push(ct.clone());
                    if let Some(schema) = media.get("schema") {
                        let (ft, _) = parse_field_type_and_nullability(schema);
                        match ft {
                            FieldType::Reference(r) => schema_ref = Some(r),
                            other => inline_schema = Some(other),
                        }
                    }
                }
            }

            responses.insert(
                code.clone(),
                ResponseContract {
                    status_code: code.clone(),
                    description,
                    schema_ref,
                    inline_schema,
                    content_types,
                },
            );
        }
    }

    EndpointContract {
        method: method.to_ascii_uppercase(),
        path: path.to_string(),
        summary,
        operation_id,
        parameters,
        request_body,
        responses,
        deprecated,
    }
}
