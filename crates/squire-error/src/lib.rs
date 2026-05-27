use thiserror::Error;

#[derive(Error, Debug)]
pub enum SquireError {
    #[error("Vision error: {0}")]
    Vision(String),

    #[error("OCR error: {0}")]
    Ocr(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Match failed: confidence {confidence:.2} below threshold {threshold:.2}")]
    MatchFailed { confidence: f64, threshold: f64 },

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("ADB error: {0}")]
    Adb(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(String),

    #[error("Task cancelled")]
    Cancelled,

    #[error("DLL not found: {0}")]
    DllNotFound(String),
}

pub type Result<T> = std::result::Result<T, SquireError>;
