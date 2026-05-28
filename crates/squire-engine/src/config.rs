use serde::{Deserialize, Serialize};
use squire_error::{Result, SquireError};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<StepDef>,
    #[serde(default)]
    pub config: TaskConfig,
    pub success_marker: Option<SuccessMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDef {
    pub action: Action,
    pub target: StepTarget,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub on_fail: FailStrategy,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "click")]
    Click,
    #[serde(rename = "wait")]
    Wait { ms: u64 },
    #[serde(rename = "swipe")]
    Swipe { to_x: i32, to_y: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepTarget {
    #[serde(rename = "template")]
    Template { path: String, threshold: Option<f64> },
    #[serde(rename = "ocr")]
    Ocr { text: String, region: Option<[i32; 4]> },
    #[serde(rename = "coordinate")]
    Coordinate { x: i32, y: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskConfig {
    #[serde(default)]
    pub fail_strategy: FailStrategy,
    #[serde(default = "default_retry")]
    pub retry_count: u32,
    #[serde(default)]
    pub params: serde_yaml::Value,
}

fn default_retry() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FailStrategy {
    #[default]
    Retry,
    Skip,
    Pause,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SuccessMarker {
    #[serde(rename = "template")]
    Template { path: String, threshold: Option<f64> },
    #[serde(rename = "ocr")]
    Ocr { text: String, region: Option<[i32; 4]> },
}

pub fn load_task(path: &Path) -> Result<TaskDefinition> {
    let content = std::fs::read_to_string(path)?;
    let task: TaskDefinition =
        serde_yaml::from_str(&content).map_err(|e| SquireError::Yaml(e.to_string()))?;
    validate_task(&task, path)?;
    Ok(task)
}

pub fn validate_task(task: &TaskDefinition, _base_path: &Path) -> Result<()> {
    if task.steps.is_empty() {
        return Err(SquireError::Config("Task has no steps".to_string()));
    }
    for (i, step) in task.steps.iter().enumerate() {
        if step.timeout_ms == 0 {
            return Err(SquireError::Config(format!(
                "Step {} has zero timeout",
                i + 1
            )));
        }
    }
    Ok(())
}

