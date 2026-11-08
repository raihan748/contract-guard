use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Safe = 0,
    Low = 5,
    Medium = 15,
    High = 25,
    Critical = 35,
}

impl Severity {
    pub fn score_weight(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Safe => write!(f, "SAFE"),
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    // Endpoints
    EndpointRemoved,
    EndpointAdded,
    EndpointParameterAddedRequired,
    EndpointParameterAddedOptional,
    EndpointParameterRemoved,
    EndpointParameterTypeChanged,
    RequestBodyBecameRequired,
    RequestBodyBecameOptional,
    RequestBodyTypeChanged,
    ResponseStatusCodeRemoved,
    ResponseStatusCodeAdded,
    ResponseTypeChanged,

    // Entities & Properties
    EntityRemoved,
    EntityAdded,
    PropertyRemoved,
    PropertyAddedRequired,
    PropertyAddedOptional,
    PropertyTypeChanged,
    PropertyRequiredTightened,
    PropertyRequiredLoosened,
    PropertyNullabilityTightened,
    PropertyNullabilityLoosened,
    EnumVariantRemoved,
    EnumVariantAdded,
}

impl ChangeKind {
    pub fn default_severity(&self) -> Severity {
        match self {
            Self::EndpointRemoved => Severity::Critical,
            Self::EntityRemoved => Severity::High,
            Self::PropertyRemoved => Severity::High,
            Self::PropertyTypeChanged => Severity::High,
            Self::PropertyAddedRequired => Severity::High,
            Self::PropertyRequiredTightened => Severity::High,
            Self::PropertyNullabilityTightened => Severity::High,
            Self::RequestBodyBecameRequired => Severity::High,
            Self::RequestBodyTypeChanged => Severity::High,
            Self::EndpointParameterAddedRequired => Severity::High,
            Self::EndpointParameterTypeChanged => Severity::High,
            Self::ResponseTypeChanged => Severity::High,

            Self::EndpointParameterRemoved => Severity::Medium,
            Self::ResponseStatusCodeRemoved => Severity::Medium,
            Self::EnumVariantRemoved => Severity::Medium,

            Self::EndpointAdded => Severity::Safe,
            Self::EntityAdded => Severity::Safe,
            Self::PropertyAddedOptional => Severity::Safe,
            Self::PropertyRequiredLoosened => Severity::Safe,
            Self::PropertyNullabilityLoosened => Severity::Safe,
            Self::EnumVariantAdded => Severity::Safe,
            Self::EndpointParameterAddedOptional => Severity::Safe,
            Self::RequestBodyBecameOptional => Severity::Safe,
            Self::ResponseStatusCodeAdded => Severity::Safe,
        }
    }

    pub fn is_breaking(&self) -> bool {
        self.default_severity() != Severity::Safe
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractChange {
    pub kind: ChangeKind,
    pub is_breaking: bool,
    pub severity: Severity,
    pub target: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
}

impl ContractChange {
    pub fn new(
        kind: ChangeKind,
        target: impl Into<String>,
        message: impl Into<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> Self {
        let severity = kind.default_severity();
        let is_breaking = kind.is_breaking();
        Self {
            kind,
            is_breaking,
            severity,
            target: target.into(),
            message: message.into(),
            old_value,
            new_value,
        }
    }
}
