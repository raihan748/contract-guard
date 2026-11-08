use std::fs;
use std::process::Command;

#[test]
fn test_cli_e2e_snapshot_and_check() {
    let baseline_spec = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Petstore API", "version": "1.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "responses": { "200": { "description": "OK" } }
                }
            }
        },
        "components": {
            "schemas": {
                "Pet": {
                    "type": "object",
                    "required": ["id", "name"],
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" }
                    }
                }
            }
        }
    }"#;

    // Mutated: id type changed from integer -> string (BREAKING)
    let mutated_spec = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Petstore API", "version": "2.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "responses": { "200": { "description": "OK" } }
                }
            }
        },
        "components": {
            "schemas": {
                "Pet": {
                    "type": "object",
                    "required": ["id", "name"],
                    "properties": {
                        "id": { "type": "string" },
                        "name": { "type": "string" }
                    }
                }
            }
        }
    }"#;

    let temp_dir = std::env::temp_dir();
    let base_file = temp_dir.join("cli_base_spec.json");
    let snap_file = temp_dir.join("cli_base_snap.json");
    let curr_file = temp_dir.join("cli_curr_spec.json");
    let report_file = temp_dir.join("cli_diff_report.json");

    fs::write(&base_file, baseline_spec).unwrap();
    fs::write(&curr_file, mutated_spec).unwrap();

    let exe_path = env!("CARGO_BIN_EXE_contract-guard");

    // 1. Snapshot command
    let snap_output = Command::new(exe_path)
        .args(["snapshot", "--spec", base_file.to_str().unwrap(), "--out", snap_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute snapshot command");

    assert!(snap_output.status.success());
    let stdout = String::from_utf8_lossy(&snap_output.stdout);
    assert!(stdout.contains("Saved canonical snapshot to"));
    assert!(snap_file.exists());

    // 2. Check command without --fail-on-breaking (should exit 0)
    let check_output = Command::new(exe_path)
        .args([
            "check",
            "--spec",
            curr_file.to_str().unwrap(),
            "--baseline",
            snap_file.to_str().unwrap(),
            "--format",
            "json",
            "--out",
            report_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute check command");

    assert!(check_output.status.success());
    assert!(report_file.exists());
    let report_content = fs::read_to_string(&report_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&report_content).unwrap();
    assert_eq!(parsed["is_breaking"], true);

    // 3. Check command with --fail-on-breaking (should exit 1)
    let check_fail_output = Command::new(exe_path)
        .args([
            "check",
            "--spec",
            curr_file.to_str().unwrap(),
            "--baseline",
            snap_file.to_str().unwrap(),
            "--fail-on-breaking",
        ])
        .output()
        .expect("Failed to execute check with --fail-on-breaking");

    assert_eq!(check_fail_output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&check_fail_output.stderr);
    assert!(stderr.contains("breaking change(s) detected with --fail-on-breaking enabled"));

    // Cleanup
    let _ = fs::remove_file(base_file);
    let _ = fs::remove_file(snap_file);
    let _ = fs::remove_file(curr_file);
    let _ = fs::remove_file(report_file);
}
