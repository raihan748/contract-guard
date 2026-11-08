use contract_guard::diff::{ChangeKind, DiffEngine, Severity};
use contract_guard::parser::parse_spec;

#[test]
fn test_diff_added_optional_property_is_safe() {
    let baseline = r#"{
        "title": "UserProfile",
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    }"#;

    let current = r#"{
        "title": "UserProfile",
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "website": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.breaking_count, 0);
    assert_eq!(result.non_breaking_count, 1);
    assert_eq!(result.changes[0].kind, ChangeKind::PropertyAddedOptional);
    assert_eq!(result.changes[0].severity, Severity::Safe);
    assert_eq!(result.risk_level, "SAFE");
}

#[test]
fn test_diff_added_property_with_default_is_safe() {
    let baseline = r#"{
        "title": "Config",
        "type": "object",
        "properties": {
            "host": { "type": "string" }
        }
    }"#;

    let current = r#"{
        "title": "Config",
        "type": "object",
        "required": ["timeout"],
        "properties": {
            "host": { "type": "string" },
            "timeout": { "type": "integer", "default": 30 }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.breaking_count, 0);
    assert_eq!(result.changes[0].kind, ChangeKind::PropertyAddedOptional);
}

#[test]
fn test_diff_added_new_endpoint_is_safe() {
    let baseline = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Store", "version": "1.0.0" },
        "paths": {
            "/products": { "get": { "responses": { "200": { "description": "list" } } } }
        }
    }"#;

    let current = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Store", "version": "1.1.0" },
        "paths": {
            "/products": { "get": { "responses": { "200": { "description": "list" } } } },
            "/products/{id}/reviews": { "get": { "responses": { "200": { "description": "reviews" } } } }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.breaking_count, 0);
    assert_eq!(result.changes[0].kind, ChangeKind::EndpointAdded);
    assert_eq!(result.changes[0].severity, Severity::Safe);
}

#[test]
fn test_diff_loosened_required_constraint_is_safe() {
    let baseline = r#"{
        "title": "Article",
        "type": "object",
        "required": ["title", "slug"],
        "properties": {
            "title": { "type": "string" },
            "slug": { "type": "string" }
        }
    }"#;

    // slug is no longer required
    let current = r#"{
        "title": "Article",
        "type": "object",
        "required": ["title"],
        "properties": {
            "title": { "type": "string" },
            "slug": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.changes[0].kind, ChangeKind::PropertyRequiredLoosened);
    assert_eq!(result.changes[0].severity, Severity::Safe);
}

#[test]
fn test_diff_loosened_nullability_is_safe() {
    let baseline = r#"{
        "title": "Employee",
        "type": "object",
        "properties": {
            "middleName": { "type": "string", "nullable": false }
        }
    }"#;

    let current = r#"{
        "title": "Employee",
        "type": "object",
        "properties": {
            "middleName": { "type": "string", "nullable": true }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.changes[0].kind, ChangeKind::PropertyNullabilityLoosened);
    assert_eq!(result.changes[0].severity, Severity::Safe);
}

#[test]
fn test_diff_added_enum_variant_is_safe() {
    let baseline = r#"{
        "title": "Priority",
        "type": "object",
        "properties": {
            "level": { "type": "string", "enum": ["LOW", "MEDIUM", "HIGH"] }
        }
    }"#;

    let current = r#"{
        "title": "Priority",
        "type": "object",
        "properties": {
            "level": { "type": "string", "enum": ["LOW", "MEDIUM", "HIGH", "CRITICAL"] }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.changes[0].kind, ChangeKind::EnumVariantAdded);
    assert_eq!(result.changes[0].new_value.as_deref(), Some("CRITICAL"));
    assert_eq!(result.changes[0].severity, Severity::Safe);
}

#[test]
fn test_diff_widening_number_type_is_safe() {
    let baseline = r#"{
        "title": "Metric",
        "type": "object",
        "properties": {
            "score": { "type": "integer" }
        }
    }"#;

    // integer -> number is a widening conversion (safe)
    let current = r#"{
        "title": "Metric",
        "type": "object",
        "properties": {
            "score": { "type": "number" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(!result.is_breaking);
    assert_eq!(result.breaking_count, 0);
}
