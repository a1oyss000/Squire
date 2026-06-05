use crate::state::TaskResult;

/// Events emitted by the engine during task execution.
///
/// These flow from the executor → scheduler → Tauri event system → UI.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// A task has started executing.
    TaskStarted(String),
    /// Executor entered a node.
    NodeEntered { task: String, node: String },
    /// Executor successfully completed a node (action executed, or leaf popped).
    NodeCompleted { task: String, node: String, hit_count: u32 },
    /// A node was skipped with a reason (disabled, max_hit, timeout, not found).
    NodeSkipped { task: String, node: String, reason: String },
    /// A task completed with its result.
    TaskCompleted(TaskResult),
    /// Full trace dump in JSONL format (emitted on task completion).
    TraceDump(String),
    /// A log message for display in the UI (level: "info", "warn", "error").
    Log { task: String, level: String, message: String },
}
