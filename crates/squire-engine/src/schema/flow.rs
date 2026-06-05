use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de;
use serde_yaml::Value;

use super::shared::IncludeRef;

// ---- NextItem ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum NextItem {
    Goto(String),
    GotoWithJumpback(String),
}

impl NextItem {
    pub fn node_id(&self) -> &str {
        match self {
            NextItem::Goto(s) | NextItem::GotoWithJumpback(s) => s.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for NextItem {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if let Some(name) = s.strip_suffix(" [jumpback]") {
            Ok(NextItem::GotoWithJumpback(name.to_string()))
        } else {
            Ok(NextItem::Goto(s))
        }
    }
}

// ---- RecognizeCondition -----------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RecognizeCondition {
    Template(TemplateCondition),
    Ocr(OcrCondition),
    Color(ColorCondition),
    And(AndCondition),
    Or(OrCondition),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateCondition {
    pub template: String,
    pub roi: Option<[i32; 4]>,
    pub order_by: Option<String>,
    pub index: Option<i32>,
}

fn deserialize_index_opt<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(d)?;
    match val {
        Value::Number(n) => Ok(n.as_i64().map(|v| v as i32)),
        Value::String(s) if s.starts_with('$') => Ok(None),
        Value::String(s) => s.parse::<i32>().map(Some).map_err(de::Error::custom),
        Value::Null => Ok(None),
        other => Err(de::Error::custom(format!("invalid index value: {:?}", other))),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OcrCondition {
    pub ocr: String,
    pub roi: Option<[i32; 4]>,
    pub replace: Option<Vec<[String; 2]>>,
    pub order_by: Option<String>,
    #[serde(default, deserialize_with = "deserialize_index_opt")]
    pub index: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorSpec {
    pub roi: [i32; 4],
    pub lower: [u8; 3],
    pub upper: [u8; 3],
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorCondition {
    pub color: ColorSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AndCondition {
    pub and: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrCondition {
    pub or: Vec<String>,
}

// ---- Action ------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Action {
    Click,
    ClickAt([i32; 2]),
    ClickOffset { offset: [i32; 4] },
    ClickRepeat { repeat: u32, interval: u32 },
    Custom(String),
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match &val {
            Value::String(s) if s == "click" => return Ok(Action::Click),
            Value::Mapping(m) => {
                if let Some(custom) = m.get("custom") {
                    let name = custom.as_str().ok_or_else(|| de::Error::custom("custom must be a string"))?;
                    return Ok(Action::Custom(name.to_string()));
                }
                if let Some(click_val) = m.get("click") {
                    return parse_click_value(click_val).map_err(de::Error::custom);
                }
            }
            _ => {}
        }
        Err(de::Error::custom(format!("unrecognized action: {:?}", val)))
    }
}

fn parse_click_value(v: &Value) -> Result<Action, String> {
    match v {
        Value::Sequence(seq) => {
            if seq.len() != 2 {
                return Err("click array must have 2 elements".into());
            }
            let x = seq[0].as_i64().ok_or("click x must be integer")? as i32;
            let y = seq[1].as_i64().ok_or("click y must be integer")? as i32;
            Ok(Action::ClickAt([x, y]))
        }
        Value::Mapping(m) => {
            if let Some(offset_val) = m.get("offset") {
                let seq = offset_val.as_sequence().ok_or("offset must be array")?;
                if seq.len() != 4 {
                    return Err("offset must have 4 elements".into());
                }
                let arr = [
                    seq[0].as_i64().ok_or("offset[0] must be integer")? as i32,
                    seq[1].as_i64().ok_or("offset[1] must be integer")? as i32,
                    seq[2].as_i64().ok_or("offset[2] must be integer")? as i32,
                    seq[3].as_i64().ok_or("offset[3] must be integer")? as i32,
                ];
                return Ok(Action::ClickOffset { offset: arr });
            }
            if let (Some(r), Some(i)) = (m.get("repeat"), m.get("interval")) {
                let repeat = r.as_u64().ok_or("repeat must be u32")? as u32;
                let interval = i.as_u64().ok_or("interval must be u32")? as u32;
                return Ok(Action::ClickRepeat { repeat, interval });
            }
            Err("unrecognized click mapping".into())
        }
        _ => Err(format!("unrecognized click value: {:?}", v)),
    }
}

// ---- PreWait -----------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct PreWait {
    pub freeze: u64,
    pub roi: Option<[i32; 4]>,
}

// ---- NodeDef -----------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct NodeDef {
    pub recognize: Option<RecognizeCondition>,
    pub action: Option<Action>,
    pub next: Option<Vec<NextItem>>,
    pub max_hit: Option<u32>,
    pub timeout: Option<u64>,
    pub pre_wait: Option<PreWait>,
    pub post_wait: Option<u64>,
}

// ---- FlowDocument ------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct FlowDocument {
    pub entry: String,
    pub includes: Option<Vec<IncludeRef>>,
    pub nodes: HashMap<String, NodeDef>,
}
