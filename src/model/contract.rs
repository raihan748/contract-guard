use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveType {
    String,
    Integer,
    Number,
    Boolean,
    Null,
    Object,
    Array,
    Any,
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Integer => write!(f, "integer"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Null => write!(f, "null"),
            Self::Object => write!(f, "object"),
            Self::Array => write!(f, "array"),
            Self::Any => write!(f, "any"),
        }
    }
}

impl PrimitiveType {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "string" => Self::String,
            "int" | "integer" | "int32" | "int64" => Self::Integer,
            "number" | "float" | "double" => Self::Number,
            "bool" | "boolean" => Self::Boolean,
            "null" => Self::Null,
            "object" => Self::Object,
            "array" => Self::Array,
            _ => Self::Any,
        }
    }

    /// Check if this type can safely accept values of `other` without breaking clients
    pub fn is_lossless_widening_of(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        // Integer -> Number is widening and non-breaking for numerical values
        matches!((self, other), (Self::Number, Self::Integer) | (Self::Any, _))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum FieldType {
    Primitive(PrimitiveType),
    Array(Box<FieldType>),
    Reference(String),
    Union(Vec<FieldType>),
    Object,
    Any,
    Unknown(String),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(p) => write!(f, "{p}"),
            Self::Array(inner) => write!(f, "Array<{inner}>"),
            Self::Reference(r) => write!(f, "${r}"),
            Self::Union(types) => {
                let joined = types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "({joined})")
            }
            Self::Object => write!(f, "object"),
            Self::Any => write!(f, "any"),
            Self::Unknown(u) => write!(f, "unknown({u})"),
        }
    }
}

impl FieldType {
    /// Determines if mutating from `self` (old) to `new_type` is compatible / non-breaking.
    pub fn is_compatible_replacement(&self, new_type: &FieldType) -> bool {
        if self == new_type {
            return true;
        }
        match (self, new_type) {
            (Self::Primitive(old_p), Self::Primitive(new_p)) => {
                new_p.is_lossless_widening_of(old_p)
            }
            (Self::Array(old_inner), Self::Array(new_inner)) => {
                old_inner.is_compatible_replacement(new_inner)
            }
            // If new type is Any, it accepts anything
            (_, Self::Any) => true,
            // If new type is a union that contains old type, it is widening (non-breaking)
            (old, Self::Union(variants)) => variants.iter().any(|v| v.is_compatible_replacement(old)),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyContract {
    pub name: String,
    pub prop_type: FieldType,
    pub nullable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, PropertyContract>,
    #[serde(default)]
    pub required_properties: BTreeSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_variants: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub additional_properties: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

impl fmt::Display for ParameterLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path => write!(f, "path"),
            Self::Query => write!(f, "query"),
            Self::Header => write!(f, "header"),
            Self::Cookie => write!(f, "cookie"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterContract {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub schema_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBodyContract {
    pub required: bool,
    #[serde(default)]
    pub content_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_schema: Option<FieldType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseContract {
    pub status_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_schema: Option<FieldType>,
    #[serde(default)]
    pub content_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointContract {
    pub method: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub parameters: Vec<ParameterContract>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBodyContract>,
    #[serde(default)]
    pub responses: BTreeMap<String, ResponseContract>,
    #[serde(default)]
    pub deprecated: bool,
}

impl EndpointContract {
    pub fn key(&self) -> String {
        format!("{} {}", self.method.to_ascii_uppercase(), self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalContract {
    pub spec_version: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub format_source: String,
    pub created_at: String,
    #[serde(default)]
    pub endpoints: BTreeMap<String, EndpointContract>,
    #[serde(default)]
    pub entities: BTreeMap<String, EntityContract>,
}

impl CanonicalContract {
    pub fn new(title: impl Into<String>, format_source: impl Into<String>) -> Self {
        Self {
            spec_version: "1.0.0".to_string(),
            title: title.into(),
            description: None,
            format_source: format_source.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            endpoints: BTreeMap::new(),
            entities: BTreeMap::new(),
        }
    }

    pub fn add_endpoint(&mut self, endpoint: EndpointContract) {
        let key = endpoint.key();
        self.endpoints.insert(key, endpoint);
    }

    pub fn add_entity(&mut self, entity: EntityContract) {
        self.entities.insert(entity.name.clone(), entity);
    }
}
