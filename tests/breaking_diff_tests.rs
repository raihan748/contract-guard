use contract_guard::diff::{ChangeKind, DiffEngine, Severity};
use contract_guard::parser::parse_spec;

#[test]
fn test_diff_removed_endpoint_is_critical_breaking() {
    let baseline = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Store API", "version": "1.0.0" },
        "paths": {
            "/items": { "get": { "responses": { "200": { "description": "ok" } } } },
            "/items/{id}": { "delete": { "responses": { "204": { "description": "deleted" } } } }
        }
    }"#;

    let current = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Store API", "version": "1.1.0" },
        "paths": {
            "/items": { "get": { "responses": { "200": { "description": "ok" } } } }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    assert_eq!(result.breaking_count, 1);
    assert_eq!(result.changes[0].kind, ChangeKind::EndpointRemoved);
    assert_eq!(result.changes[0].severity, Severity::Critical);
    assert_eq!(result.risk_level, "CRITICAL");
}

#[test]
fn test_diff_removed_property_is_high_breaking() {
    let baseline = r#"{
        "title": "User",
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "email": { "type": "string" }
        }
    }"#;

    let current = r#"{
        "title": "User",
        "type": "object",
        "properties": {
            "id": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::PropertyRemoved).unwrap();
    assert_eq!(change.severity, Severity::High);
    assert!(change.target.contains("User.email"));
}

#[test]
fn test_diff_changed_property_type_is_high_breaking() {
    let baseline = r#"{
        "title": "Account",
        "type": "object",
        "properties": {
            "balance": { "type": "integer" }
        }
    }"#;

    // Mutated: integer -> string
    let current = r#"{
        "title": "Account",
        "type": "object",
        "properties": {
            "balance": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::PropertyTypeChanged).unwrap();
    assert_eq!(change.severity, Severity::High);
    assert!(change.message.contains("integer"));
    assert!(change.message.contains("string"));
}

#[test]
fn test_diff_newly_required_property_without_default_is_breaking() {
    let baseline = r#"{
        "title": "Order",
        "type": "object",
        "properties": {
            "orderId": { "type": "string" }
        }
    }"#;

    // Added shippingAddress as required without default
    let current = r#"{
        "title": "Order",
        "type": "object",
        "required": ["shippingAddress"],
        "properties": {
            "orderId": { "type": "string" },
            "shippingAddress": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::PropertyAddedRequired).unwrap();
    assert_eq!(change.severity, Severity::High);
}

#[test]
fn test_diff_tightened_required_constraint_is_breaking() {
    let baseline = r#"{
        "title": "UserProfile",
        "type": "object",
        "properties": {
            "phone": { "type": "string" }
        }
    }"#;

    // phone was optional, now made required without default
    let current = r#"{
        "title": "UserProfile",
        "type": "object",
        "required": ["phone"],
        "properties": {
            "phone": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::PropertyRequiredTightened).unwrap();
    assert_eq!(change.severity, Severity::High);
}

#[test]
fn test_diff_tightened_nullability_is_breaking() {
    let baseline = r#"{
        "title": "Profile",
        "type": "object",
        "properties": {
            "avatarUrl": { "type": "string", "nullable": true }
        }
    }"#;

    let current = r#"{
        "title": "Profile",
        "type": "object",
        "properties": {
            "avatarUrl": { "type": "string", "nullable": false }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::PropertyNullabilityTightened).unwrap();
    assert_eq!(change.severity, Severity::High);
}

#[test]
fn test_diff_removed_enum_variant_is_medium_breaking() {
    let baseline = r#"{
        "title": "Subscription",
        "type": "object",
        "properties": {
            "plan": {
                "type": "string",
                "enum": ["FREE", "PRO", "ENTERPRISE", "LIFETIME"]
            }
        }
    }"#;

    // LIFETIME removed
    let current = r#"{
        "title": "Subscription",
        "type": "object",
        "properties": {
            "plan": {
                "type": "string",
                "enum": ["FREE", "PRO", "ENTERPRISE"]
            }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    let change = result.changes.iter().find(|c| c.kind == ChangeKind::EnumVariantRemoved).unwrap();
    assert_eq!(change.severity, Severity::Medium);
    assert_eq!(change.old_value.as_deref(), Some("LIFETIME"));
}

#[test]
fn test_diff_endpoint_parameter_and_request_body_breaking() {
    let baseline = r#"{
        "openapi": "3.0.0",
        "info": { "title": "API", "version": "1.0.0" },
        "paths": {
            "/search": {
                "post": {
                    "requestBody": {
                        "required": false
                    },
                    "responses": {
                        "200": { "description": "OK" },
                        "404": { "description": "Not Found" }
                    }
                }
            }
        }
    }"#;

    // Changed: requestBody is now required, new required query param 'apiKey' added, and 404 response removed
    let current = r#"{
        "openapi": "3.0.0",
        "info": { "title": "API", "version": "2.0.0" },
        "paths": {
            "/search": {
                "post": {
                    "parameters": [
                        { "name": "apiKey", "in": "query", "required": true }
                    ],
                    "requestBody": {
                        "required": true
                    },
                    "responses": {
                        "200": { "description": "OK" }
                    }
                }
            }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    assert!(result.is_breaking);
    assert!(result.changes.iter().any(|c| c.kind == ChangeKind::RequestBodyBecameRequired));
    assert!(result.changes.iter().any(|c| c.kind == ChangeKind::EndpointParameterAddedRequired));
    assert!(result.changes.iter().any(|c| c.kind == ChangeKind::ResponseStatusCodeRemoved));
}
