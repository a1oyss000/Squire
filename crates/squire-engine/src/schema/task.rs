use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

// ---- OptionDef ---------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OptionDef {
    Select(SelectOption),
    Checkbox(CheckboxOption),
    Switch(SwitchOption),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectOption {
    pub default: String,
    pub cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckboxOption {
    pub default: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwitchOption {
    pub default: bool,
    pub when_config: Option<HashMap<String, String>>,
}

// ---- ResourceValue -----------------------------------------------------------

/// A resource value can be a single string, a list of strings, or a number.
#[derive(Debug, Clone)]
pub enum ResourceValue {
    Single(String),
    List(Vec<String>),
    Number(i64),
}

impl<'de> Deserialize<'de> for ResourceValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::String(s) => Ok(ResourceValue::Single(s)),
            Value::Sequence(seq) => {
                let mut items = Vec::new();
                for v in seq {
                    match v {
                        Value::String(s) => items.push(s),
                        other => items.push(other.as_str().unwrap_or("").to_string()),
                    }
                }
                Ok(ResourceValue::List(items))
            }
            Value::Number(n) => Ok(ResourceValue::Number(
                n.as_i64().unwrap_or(0),
            )),
            _ => Ok(ResourceValue::Single(String::new())),
        }
    }
}

// ---- Binding -----------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Binding {
    pub nodes: Vec<String>,
    pub enabled_by: String,
}

// ---- TaskDocument ------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct TaskDocument {
    pub name: String,
    pub label: String,
    pub flow: String,
    pub group: Option<Vec<String>>,
    pub options: Option<HashMap<String, OptionDef>>,
    /// resources: keyed by option case value, then by variable name
    pub resources: Option<HashMap<String, HashMap<String, ResourceValue>>>,
    pub bindings: Option<Vec<Binding>>,
}
