use contract_guard::diff::DiffEngine;
use contract_guard::parser::parse_spec;
use contract_guard::reporter::{render_report, OutputFormat};
use contract_guard::snapshot::{create_snapshot, load_snapshot, save_snapshot};
use std::fs;

#[test]
fn test_snapshot_roundtrip_fidelity() {
    let spec = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Billing API", "version": "1.0.0" },
        "paths": {
            "/invoices": {
                "get": {
                    "responses": {
                        "200": { "description": "list invoices" }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "Invoice": {
                    "type": "object",
                    "required": ["id", "amount"],
                    "properties": {
                        "id": { "type": "string" },
                        "amount": { "type": "number" },
                        "paid": { "type": "boolean" }
                    }
                }
            }
        }
    }"#;

    let temp_spec_path = std::env::temp_dir().join("test_billing_spec.json");
    let temp_snap_path = std::env::temp_dir().join("test_billing_snap.json");

    fs::write(&temp_spec_path, spec).unwrap();

    let contract = create_snapshot(&temp_spec_path).unwrap();
    save_snapshot(&contract, &temp_snap_path).unwrap();

    let loaded = load_snapshot(&temp_snap_path).unwrap();
    assert_eq!(loaded.title, "Billing API");
    assert_eq!(loaded.endpoints.len(), 1);
    assert_eq!(loaded.entities.len(), 1);

    // Diff against itself should be 100% clean
    let result = DiffEngine::diff(&contract, &loaded);
    assert!(!result.is_breaking);
    assert_eq!(result.changes.len(), 0);
    assert_eq!(result.risk_level, "SAFE");
    assert_eq!(result.severity_score, 0);

    let _ = fs::remove_file(temp_spec_path);
    let _ = fs::remove_file(temp_snap_path);
}

#[test]
fn test_diff_reporters_render_valid_formats() {
    let baseline = r#"{
        "title": "Auth",
        "type": "object",
        "properties": {
            "token": { "type": "string" }
        }
    }"#;

    let current = r#"{
        "title": "Auth",
        "type": "object",
        "properties": {
            "token": { "type": "integer" },
            "refreshToken": { "type": "string" }
        }
    }"#;

    let base = parse_spec(baseline).unwrap();
    let curr = parse_spec(current).unwrap();
    let result = DiffEngine::diff(&base, &curr);

    // 1. JSON
    let json_output = render_report(&result, OutputFormat::Json);
    let parsed_json: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed_json["is_breaking"], true);
    assert_eq!(parsed_json["breaking_count"], 1);
    assert_eq!(parsed_json["non_breaking_count"], 1);

    // 2. Markdown
    let md_output = render_report(&result, OutputFormat::Markdown);
    assert!(md_output.contains("# Contract Regression Report"));
    assert!(md_output.contains("BREAKING CONTRACT REGRESSION DETECTED"));
    assert!(md_output.contains("| `HIGH` | 🚨 YES |"));

    // 3. Table
    let table_output = render_report(&result, OutputFormat::Table);
    assert!(table_output.contains("CONTRACT-GUARD REPORT"));
    assert!(table_output.contains("BREAKING"));
}
