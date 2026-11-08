use crate::diff::change::{ChangeKind, ContractChange, Severity};
use crate::model::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffResult {
    pub baseline_title: String,
    pub current_title: String,
    pub is_breaking: bool,
    pub breaking_count: usize,
    pub non_breaking_count: usize,
    pub severity_score: u32,
    pub risk_level: String,
    pub changes: Vec<ContractChange>,
}

pub struct DiffEngine;

impl DiffEngine {
    pub fn diff(baseline: &CanonicalContract, current: &CanonicalContract) -> DiffResult {
        let mut changes = Vec::new();

        // 1. Diff Endpoints
        Self::diff_endpoints(baseline, current, &mut changes);

        // 2. Diff Entities / Models
        Self::diff_entities(baseline, current, &mut changes);

        let breaking_count = changes.iter().filter(|c| c.is_breaking).count();
        let non_breaking_count = changes.iter().filter(|c| !c.is_breaking).count();

        let total_weight: u32 = changes
            .iter()
            .filter(|c| c.is_breaking)
            .map(|c| c.severity.score_weight())
            .sum();
        let severity_score = total_weight.min(100);

        let has_critical = changes.iter().any(|c| c.severity == Severity::Critical);
        let risk_level = if has_critical || severity_score >= 60 {
            "CRITICAL".to_string()
        } else if severity_score >= 35 {
            "HIGH".to_string()
        } else if severity_score >= 15 {
            "MEDIUM".to_string()
        } else if severity_score > 0 {
            "LOW".to_string()
        } else {
            "SAFE".to_string()
        };

        DiffResult {
            baseline_title: baseline.title.clone(),
            current_title: current.title.clone(),
            is_breaking: breaking_count > 0,
            breaking_count,
            non_breaking_count,
            severity_score,
            risk_level,
            changes,
        }
    }

    fn diff_endpoints(
        baseline: &CanonicalContract,
        current: &CanonicalContract,
        changes: &mut Vec<ContractChange>,
    ) {
        // Endpoints in baseline
        for (key, base_ep) in &baseline.endpoints {
            match current.endpoints.get(key) {
                None => {
                    changes.push(ContractChange::new(
                        ChangeKind::EndpointRemoved,
                        format!("Endpoint `{key}`"),
                        format!("Endpoint `{key}` was removed from API contract"),
                        Some(key.clone()),
                        None,
                    ));
                }
                Some(curr_ep) => {
                    Self::diff_single_endpoint(key, base_ep, curr_ep, changes);
                }
            }
        }

        // Endpoints added in current
        for (key, _curr_ep) in &current.endpoints {
            if !baseline.endpoints.contains_key(key) {
                changes.push(ContractChange::new(
                    ChangeKind::EndpointAdded,
                    format!("Endpoint `{key}`"),
                    format!("New endpoint `{key}` was added"),
                    None,
                    Some(key.clone()),
                ));
            }
        }
    }

    fn diff_single_endpoint(
        key: &str,
        base_ep: &EndpointContract,
        curr_ep: &EndpointContract,
        changes: &mut Vec<ContractChange>,
    ) {
        // Diff parameters
        for base_p in &base_ep.parameters {
            match curr_ep.parameters.iter().find(|p| p.name == base_p.name && p.location == base_p.location) {
                None => {
                    changes.push(ContractChange::new(
                        ChangeKind::EndpointParameterRemoved,
                        format!("Endpoint `{key}` param `{}` ({})", base_p.name, base_p.location),
                        format!("Parameter `{}` was removed from endpoint `{key}`", base_p.name),
                        Some(base_p.schema_type.to_string()),
                        None,
                    ));
                }
                Some(curr_p) => {
                    if !base_p.schema_type.is_compatible_replacement(&curr_p.schema_type) {
                        changes.push(ContractChange::new(
                            ChangeKind::EndpointParameterTypeChanged,
                            format!("Endpoint `{key}` param `{}` ({})", base_p.name, base_p.location),
                            format!(
                                "Parameter `{}` type changed from `{}` to `{}`",
                                base_p.name, base_p.schema_type, curr_p.schema_type
                            ),
                            Some(base_p.schema_type.to_string()),
                            Some(curr_p.schema_type.to_string()),
                        ));
                    }
                }
            }
        }

        for curr_p in &curr_ep.parameters {
            if !base_ep.parameters.iter().any(|p| p.name == curr_p.name && p.location == curr_p.location) {
                if curr_p.required && curr_p.default_value.is_none() {
                    changes.push(ContractChange::new(
                        ChangeKind::EndpointParameterAddedRequired,
                        format!("Endpoint `{key}` param `{}` ({})", curr_p.name, curr_p.location),
                        format!(
                            "New required parameter `{}` added without default value to endpoint `{key}`",
                            curr_p.name
                        ),
                        None,
                        Some(curr_p.schema_type.to_string()),
                    ));
                } else {
                    changes.push(ContractChange::new(
                        ChangeKind::EndpointParameterAddedOptional,
                        format!("Endpoint `{key}` param `{}` ({})", curr_p.name, curr_p.location),
                        format!("New optional parameter `{}` added to endpoint `{key}`", curr_p.name),
                        None,
                        Some(curr_p.schema_type.to_string()),
                    ));
                }
            }
        }

        // Diff Request Body
        match (&base_ep.request_body, &curr_ep.request_body) {
            (Some(b_rb), Some(c_rb)) => {
                if !b_rb.required && c_rb.required {
                    changes.push(ContractChange::new(
                        ChangeKind::RequestBodyBecameRequired,
                        format!("Endpoint `{key}` requestBody"),
                        format!("Request body for `{key}` changed from optional to required"),
                        Some("optional".to_string()),
                        Some("required".to_string()),
                    ));
                } else if b_rb.required && !c_rb.required {
                    changes.push(ContractChange::new(
                        ChangeKind::RequestBodyBecameOptional,
                        format!("Endpoint `{key}` requestBody"),
                        format!("Request body for `{key}` relaxed to optional"),
                        Some("required".to_string()),
                        Some("optional".to_string()),
                    ));
                }

                if b_rb.schema_ref != c_rb.schema_ref && b_rb.schema_ref.is_some() && c_rb.schema_ref.is_some() {
                    changes.push(ContractChange::new(
                        ChangeKind::RequestBodyTypeChanged,
                        format!("Endpoint `{key}` requestBody schema"),
                        format!(
                            "Request body schema reference changed from `{:?}` to `{:?}`",
                            b_rb.schema_ref, c_rb.schema_ref
                        ),
                        b_rb.schema_ref.clone(),
                        c_rb.schema_ref.clone(),
                    ));
                }
            }
            (None, Some(c_rb)) => {
                if c_rb.required {
                    changes.push(ContractChange::new(
                        ChangeKind::RequestBodyBecameRequired,
                        format!("Endpoint `{key}` requestBody"),
                        format!("New required request body added to endpoint `{key}`"),
                        None,
                        Some("required".to_string()),
                    ));
                }
            }
            _ => {}
        }

        // Diff Responses
        for (code, base_resp) in &base_ep.responses {
            match curr_ep.responses.get(code) {
                None => {
                    changes.push(ContractChange::new(
                        ChangeKind::ResponseStatusCodeRemoved,
                        format!("Endpoint `{key}` response `{code}`"),
                        format!("Documented HTTP response `{code}` removed from `{key}`"),
                        Some(code.clone()),
                        None,
                    ));
                }
                Some(curr_resp) => {
                    if base_resp.schema_ref.is_some()
                        && curr_resp.schema_ref.is_some()
                        && base_resp.schema_ref != curr_resp.schema_ref
                    {
                        changes.push(ContractChange::new(
                            ChangeKind::ResponseTypeChanged,
                            format!("Endpoint `{key}` response `{code}` schema"),
                            format!(
                                "Response schema reference for `{code}` changed from `{:?}` to `{:?}`",
                                base_resp.schema_ref, curr_resp.schema_ref
                            ),
                            base_resp.schema_ref.clone(),
                            curr_resp.schema_ref.clone(),
                        ));
                    }
                }
            }
        }

        for (code, _) in &curr_ep.responses {
            if !base_ep.responses.contains_key(code) {
                changes.push(ContractChange::new(
                    ChangeKind::ResponseStatusCodeAdded,
                    format!("Endpoint `{key}` response `{code}`"),
                    format!("New response code `{code}` documented for endpoint `{key}`"),
                    None,
                    Some(code.clone()),
                ));
            }
        }
    }

    fn diff_entities(
        baseline: &CanonicalContract,
        current: &CanonicalContract,
        changes: &mut Vec<ContractChange>,
    ) {
        // Check baseline entities
        for (name, base_ent) in &baseline.entities {
            match current.entities.get(name) {
                None => {
                    changes.push(ContractChange::new(
                        ChangeKind::EntityRemoved,
                        format!("Entity `{name}`"),
                        format!("Model/Entity `{name}` was removed from schema"),
                        Some(name.clone()),
                        None,
                    ));
                }
                Some(curr_ent) => {
                    Self::diff_single_entity(name, base_ent, curr_ent, changes);
                }
            }
        }

        // Check new entities in current
        for (name, _curr_ent) in &current.entities {
            if !baseline.entities.contains_key(name) {
                changes.push(ContractChange::new(
                    ChangeKind::EntityAdded,
                    format!("Entity `{name}`"),
                    format!("New Model/Entity `{name}` was added to schema"),
                    None,
                    Some(name.clone()),
                ));
            }
        }
    }

    fn diff_single_entity(
        entity_name: &str,
        base_ent: &EntityContract,
        curr_ent: &EntityContract,
        changes: &mut Vec<ContractChange>,
    ) {
        // Properties in baseline
        for (prop_name, base_prop) in &base_ent.properties {
            match curr_ent.properties.get(prop_name) {
                None => {
                    changes.push(ContractChange::new(
                        ChangeKind::PropertyRemoved,
                        format!("Property `{entity_name}.{prop_name}`"),
                        format!("Field `{prop_name}` was removed from entity `{entity_name}`"),
                        Some(base_prop.prop_type.to_string()),
                        None,
                    ));
                }
                Some(curr_prop) => {
                    // 1. Check type change
                    if !base_prop.prop_type.is_compatible_replacement(&curr_prop.prop_type) {
                        changes.push(ContractChange::new(
                            ChangeKind::PropertyTypeChanged,
                            format!("Property `{entity_name}.{prop_name}`"),
                            format!(
                                "Field `{prop_name}` type changed incompatibly from `{}` to `{}`",
                                base_prop.prop_type, curr_prop.prop_type
                            ),
                            Some(base_prop.prop_type.to_string()),
                            Some(curr_prop.prop_type.to_string()),
                        ));
                    }

                    // 2. Check required tightening / loosening
                    let base_req = base_ent.required_properties.contains(prop_name);
                    let curr_req = curr_ent.required_properties.contains(prop_name);

                    if !base_req && curr_req && curr_prop.default_value.is_none() {
                        changes.push(ContractChange::new(
                            ChangeKind::PropertyRequiredTightened,
                            format!("Property `{entity_name}.{prop_name}`"),
                            format!(
                                "Existing field `{prop_name}` in `{entity_name}` became required without default value"
                            ),
                            Some("optional".to_string()),
                            Some("required".to_string()),
                        ));
                    } else if base_req && !curr_req {
                        changes.push(ContractChange::new(
                            ChangeKind::PropertyRequiredLoosened,
                            format!("Property `{entity_name}.{prop_name}`"),
                            format!("Required field `{prop_name}` in `{entity_name}` relaxed to optional"),
                            Some("required".to_string()),
                            Some("optional".to_string()),
                        ));
                    }

                    // 3. Check nullability tightening / loosening
                    if base_prop.nullable && !curr_prop.nullable {
                        changes.push(ContractChange::new(
                            ChangeKind::PropertyNullabilityTightened,
                            format!("Property `{entity_name}.{prop_name}`"),
                            format!("Nullable field `{prop_name}` in `{entity_name}` tightened to non-nullable"),
                            Some("nullable: true".to_string()),
                            Some("nullable: false".to_string()),
                        ));
                    } else if !base_prop.nullable && curr_prop.nullable {
                        changes.push(ContractChange::new(
                            ChangeKind::PropertyNullabilityLoosened,
                            format!("Property `{entity_name}.{prop_name}`"),
                            format!("Field `{prop_name}` in `{entity_name}` loosened to allow null"),
                            Some("nullable: false".to_string()),
                            Some("nullable: true".to_string()),
                        ));
                    }

                    // 4. Check property-level enums
                    if let (Some(b_enums), Some(c_enums)) = (&base_prop.enum_values, &curr_prop.enum_values) {
                        for old_v in b_enums {
                            if !c_enums.contains(old_v) {
                                changes.push(ContractChange::new(
                                    ChangeKind::EnumVariantRemoved,
                                    format!("Property `{entity_name}.{prop_name}` enum"),
                                    format!(
                                        "Enum variant `{old_v}` removed from property `{entity_name}.{prop_name}`"
                                    ),
                                    Some(old_v.clone()),
                                    None,
                                ));
                            }
                        }
                        for new_v in c_enums {
                            if !b_enums.contains(new_v) {
                                changes.push(ContractChange::new(
                                    ChangeKind::EnumVariantAdded,
                                    format!("Property `{entity_name}.{prop_name}` enum"),
                                    format!(
                                        "Enum variant `{new_v}` added to property `{entity_name}.{prop_name}`"
                                    ),
                                    None,
                                    Some(new_v.clone()),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Properties added in current
        for (prop_name, curr_prop) in &curr_ent.properties {
            if !base_ent.properties.contains_key(prop_name) {
                let is_required = curr_ent.required_properties.contains(prop_name);
                if is_required && curr_prop.default_value.is_none() {
                    changes.push(ContractChange::new(
                        ChangeKind::PropertyAddedRequired,
                        format!("Property `{entity_name}.{prop_name}`"),
                        format!(
                            "Newly added field `{prop_name}` on `{entity_name}` is required without default"
                        ),
                        None,
                        Some(curr_prop.prop_type.to_string()),
                    ));
                } else {
                    changes.push(ContractChange::new(
                        ChangeKind::PropertyAddedOptional,
                        format!("Property `{entity_name}.{prop_name}`"),
                        format!("New optional field `{prop_name}` added to `{entity_name}`"),
                        None,
                        Some(curr_prop.prop_type.to_string()),
                    ));
                }
            }
        }

        // Check Entity-level enums
        if let (Some(b_enums), Some(c_enums)) = (&base_ent.enum_variants, &curr_ent.enum_variants) {
            for old_v in b_enums {
                if !c_enums.contains(old_v) {
                    changes.push(ContractChange::new(
                        ChangeKind::EnumVariantRemoved,
                        format!("Entity `{entity_name}` enum"),
                        format!("Enum variant `{old_v}` removed from `{entity_name}`"),
                        Some(old_v.clone()),
                        None,
                    ));
                }
            }
            for new_v in c_enums {
                if !b_enums.contains(new_v) {
                    changes.push(ContractChange::new(
                        ChangeKind::EnumVariantAdded,
                        format!("Entity `{entity_name}` enum"),
                        format!("Enum variant `{new_v}` added to `{entity_name}`"),
                        None,
                        Some(new_v.clone()),
                    ));
                }
            }
        }
    }
}
