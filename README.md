# contract-guard

[![crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io)
[![Documentation](https://docs.rs/contract-guard/badge.svg)](https://docs.rs/contract-guard)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![CI](https://img.shields.io/badge/CI-100%25%20Passing-brightgreen.svg)]()
[![Zero Warnings](https://img.shields.io/badge/Compiler-0%20Warnings-brightgreen.svg)]()

> **API Schema & Contract Regression Snapshotter and Semantic Breaking Change Differ**  
> Prevent accidental API contract regressions before they break downstream mobile, web, and microservice clients.

---

## 🚨 The Vibecoding Dilemma

AI coding assistants enable developers to refactor backend models, DTOs, and controllers at lightning speed. However, AI copilots routinely introduce subtle, catastrophic contract regressions:
- Silently renaming or removing DTO fields
- Changing numeric IDs from `integer` to `string` or vice versa
- Dropping endpoints or mutating response HTTP status codes
- Tightening nullability constraints (`null` rejected by deserializer)
- Adding required fields without default fallback values
- Truncating enum variants expected by mobile clients

Downstream native mobile apps (iOS/Android), single-page apps (React/Vue), and partner webhooks crash in production upon deserializing mutated payloads.

**`contract-guard`** intercepts these regressions at the source. It extracts canonical snapshots from **OpenAPI 3.0/3.1** and **JSON Schema (draft-07, 2020-12)** definitions, semantically diffs current schemas against production baselines, and enforces strict backward-compatibility gates in CI/CD pipelines.

---

## 🏛️ Architecture & Semantic Diff Flow

```
┌─────────────────────────┐          ┌─────────────────────────┐
│ OpenAPI 3.0 / 3.1 Spec  │          │ JSON Schema (07 / 2020) │
│       (JSON/YAML)       │          │       (JSON/YAML)       │
└────────────┬────────────┘          └────────────┬────────────┘
             │                                    │
             ▼                                    ▼
┌──────────────────────────────────────────────────────────────┐
│              Schema Extractor & Normalizer                   │
│   - Resolves $ref pointers and entity definitions            │
│   - Normalizes union types, arrays, enums, & nullability     │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                  Canonical Contract Snapshot                 │
│         Endpoints: {Method, Path, Params, Body, Resps}       │
│         Entities:  {Name, Properties, Required, Enums}       │
└──────────────┬────────────────────────────────┬──────────────┘
               │                                │
      [Baseline Snapshot]              [Current Spec Branch]
               │                                │
               └───────────────┬────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                 Semantic Diff & Rule Engine                  │
│   - Breaking: endpoint/field removals, type shifts, enums    │
│   - Safe: optional additions, widening, relaxed constraints  │
│   - Computes weighted severity score & impact level          │
└──────────────────────────────┬───────────────────────────────┘
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
┌────────────────────────┐            ┌────────────────────────┐
│  Multi-Format Reporter │            │    CI/CD Gatekeeper    │
│  - Rich Terminal Table │            │  --fail-on-breaking    │
│  - Machine JSON        │            │  Exit Code: 0 (Safe)   │
│  - CI Markdown / PR    │            │  Exit Code: 1 (Broke)  │
└────────────────────────┘            └────────────────────────┘
```

---

## ⚖️ Contract Diff Classification Matrix

`contract-guard` categorizes every contract mutation into **BREAKING** or **SAFE (Non-Breaking)** with deterministic severity weighting:

| Mutation Category | Specific Change | Severity | Classification | Impact on Downstream Clients |
| :--- | :--- | :---: | :---: | :--- |
| **Endpoint** | Endpoint removed | `CRITICAL` | 🚨 **BREAKING** | Client receives 404 Not Found |
| **Endpoint** | Endpoint added | `SAFE` | 🛡️ **SAFE** | Existing clients unaffected |
| **Endpoint** | New required query/path param (no default) | `HIGH` | 🚨 **BREAKING** | Client requests rejected with 400 Bad Request |
| **Endpoint** | New optional query param | `SAFE` | 🛡️ **SAFE** | Backward-compatible parameter |
| **Endpoint** | Existing parameter removed | `MEDIUM` | 🚨 **BREAKING** | Client param silently ignored or route fails |
| **Endpoint** | Parameter type changed incompatibly | `HIGH` | 🚨 **BREAKING** | Route binding fails with 400 |
| **Request Body** | Body changed from optional to required | `HIGH` | 🚨 **BREAKING** | Existing payload-less calls fail |
| **Response** | Documented response status code removed | `MEDIUM` | 🚨 **BREAKING** | Client status handlers break |
| **Entity** | Model/Entity removed from spec | `HIGH` | 🚨 **BREAKING** | Dependent deserializers fail |
| **Property** | Existing field removed from model | `HIGH` | 🚨 **BREAKING** | Client parser crashes (KeyNotFound) |
| **Property** | Field type changed (e.g. `int` → `string`) | `HIGH` | 🚨 **BREAKING** | Deserialization type mismatch crash |
| **Property** | Widening numeric type (`int` → `number`) | `SAFE` | 🛡️ **SAFE** | Value fits in client numeric container |
| **Property** | Newly required field added (no default) | `HIGH` | 🚨 **BREAKING** | Payload validation rejects older clients |
| **Property** | New optional field added | `SAFE` | 🛡️ **SAFE** | Backward-compatible model extension |
| **Property** | Optional field tightened to required | `HIGH` | 🚨 **BREAKING** | Older clients omitting field will fail |
| **Property** | Required field loosened to optional | `SAFE` | 🛡️ **SAFE** | Client is already sending the field |
| **Property** | Nullable field tightened to non-nullable | `HIGH` | 🚨 **BREAKING** | Null values now cause schema failure |
| **Property** | Non-nullable field loosened to nullable | `SAFE` | 🛡️ **SAFE** | Deserializer handles null gracefully |
| **Enum** | Variant removed from enum | `MEDIUM` | 🚨 **BREAKING** | Exhaustive pattern match crashes client |
| **Enum** | Variant added to enum | `SAFE` | 🛡️ **SAFE** | Existing enum values still valid |

---

## ⚡ Installation

### Build from source:
```bash
git clone https://github.com/raihan748/contract-guard.git
cd contract-guard
cargo build --release
```
The compiled binary is available at `target/release/contract-guard`.

---

## 🚀 CLI Usage & Workflows

### 1. Snapshot Production API Contract
Capture a canonical baseline snapshot of your production API:
```bash
contract-guard snapshot --spec ./api/openapi.yaml --out ./contracts/baseline.json
```
```text
📸 Capturing API contract snapshot from: ./api/openapi.yaml
✅ Saved canonical snapshot to: ./contracts/baseline.json (Endpoints: 42, Entities: 18)
```

### 2. Verify PR Branch Against Baseline
Run regression verification on a pull request:
```bash
contract-guard check --spec ./api/openapi.yaml --baseline ./contracts/baseline.json
```

### 3. Enforce Strict CI/CD Gate
Fail the pipeline immediately if any breaking changes are detected:
```bash
contract-guard check \
  --spec ./api/openapi.yaml \
  --baseline ./contracts/baseline.json \
  --fail-on-breaking \
  --format table
```
If breaking changes exist, the process exits with **Code 1**.

### 4. Output Formats

#### Terminal Table (Default):
```
================================================================================
                             CONTRACT-GUARD REPORT                              
================================================================================
 Status:       🚨 BREAKING CONTRACT REGRESSION DETECTED
 Baseline:     Payment Service v1.0.0
 Current:      Payment Service v2.0.0-rc1
 Risk Level:   HIGH (Score: 50/100)
 Impact:       2 breaking, 1 backward-compatible mutations
--------------------------------------------------------------------------------
SEVERITY   BREAKING   TARGET                           CHANGE DESCRIPTION
--------------------------------------------------------------------------------
HIGH       YES        Property `Charge.amount`         Field `amount` type changed incompatibly from `integer` to `string`
HIGH       YES        Property `Charge.receipt_email`  Existing field `receipt_email` in `Charge` became required without default value
SAFE       NO         Property `Charge.metadata`       New optional field `metadata` added to `Charge`
================================================================================
```

#### Machine-Readable JSON:
```bash
contract-guard check --spec ./spec.json --baseline ./base.json --format json --out ./report.json
```
```json
{
  "baseline_title": "Payment Service v1.0.0",
  "current_title": "Payment Service v2.0.0-rc1",
  "is_breaking": true,
  "breaking_count": 2,
  "non_breaking_count": 1,
  "severity_score": 50,
  "risk_level": "HIGH",
  "changes": [
    {
      "kind": "property_type_changed",
      "is_breaking": true,
      "severity": "HIGH",
      "target": "Property `Charge.amount`",
      "message": "Field `amount` type changed incompatibly from `integer` to `string`",
      "old_value": "integer",
      "new_value": "string"
    }
  ]
}
```

#### GitHub PR Markdown Summary:
```bash
contract-guard check --spec ./spec.json --baseline ./base.json --format markdown --out ./pr_comment.md
```

---

## 🤖 GitHub Actions Workflow

Add this automated contract safety check to `.github/workflows/api-contract-guard.yml`:

```yaml
name: API Contract Guard

on:
  pull_request:
    paths:
      - 'api/**'
      - 'schemas/**'

jobs:
  contract-check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout PR Branch
        uses: actions/checkout@v4

      - name: Checkout Baseline Contract
        uses: actions/checkout@v4
        with:
          ref: main
          path: .baseline

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build contract-guard
        run: cargo build --release

      - name: Verify API Backward Compatibility
        run: |
          ./target/release/contract-guard check \
            --spec ./api/openapi.yaml \
            --baseline .baseline/contracts/baseline.json \
            --fail-on-breaking \
            --format markdown \
            --out ./contract-report.md

      - name: Comment on PR
        if: always()
        uses: marocchino/sticky-pull-request-comment@v2
        with:
          path: ./contract-report.md
```

---

## 🧪 Automated Test Suite

`contract-guard` features a rigorous automated test suite with 100% test pass rate and zero compiler warnings:

```bash
cargo test
```

```text
running 8 tests (tests/breaking_diff_tests.rs)
test test_diff_removed_enum_variant_is_medium_breaking ... ok
test test_diff_newly_required_property_without_default_is_breaking ... ok
test test_diff_tightened_required_constraint_is_breaking ... ok
test test_diff_tightened_nullability_is_breaking ... ok
test test_diff_removed_property_is_high_breaking ... ok
test test_diff_endpoint_parameter_and_request_body_breaking ... ok
test test_diff_changed_property_type_is_high_breaking ... ok
test test_diff_removed_endpoint_is_critical_breaking ... ok
test result: ok. 8 passed; 0 failed

running 7 tests (tests/non_breaking_diff_tests.rs)
test test_diff_added_optional_property_is_safe ... ok
test test_diff_loosened_nullability_is_safe ... ok
test test_diff_added_property_with_default_is_safe ... ok
test test_diff_loosened_required_constraint_is_safe ... ok
test test_diff_added_enum_variant_is_safe ... ok
test test_diff_widening_number_type_is_safe ... ok
test test_diff_added_new_endpoint_is_safe ... ok
test result: ok. 7 passed; 0 failed

running 4 tests (tests/spec_parser_tests.rs)
test test_parse_json_schema_2020_12 ... ok
test test_parse_json_schema_draft7 ... ok
test test_parse_openapi_3_0_json ... ok
test test_parse_openapi_3_1_yaml ... ok
test result: ok. 4 passed; 0 failed

running 2 tests (tests/snapshot_cli_tests.rs)
test test_diff_reporters_render_valid_formats ... ok
test test_snapshot_roundtrip_fidelity ... ok
test result: ok. 2 passed; 0 failed

running 1 test (tests/cli_e2e_tests.rs)
test test_cli_e2e_snapshot_and_check ... ok
test result: ok. 1 passed; 0 failed
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
