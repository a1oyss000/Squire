use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, watch};
use tracing::info;

use crate::executor::{Executor, ExecutorConfig, ExecutorStatus, LoadedTask as ExecTask};
use crate::loader;
use crate::state::{TaskResult, TaskState};
use crate::vision_impl::WgcVisionProvider;
use squire_input::InputBackend;

// Re-export for backward compatibility (src-tauri depends on these)
pub use crate::event::EngineEvent;

pub enum EngineCommand {
    Start,
    Cancel,
}

pub struct EngineHandle {
    pub cmd_tx: mpsc::Sender<EngineCommand>,
    pub event_rx: mpsc::Receiver<EngineEvent>,
}

pub fn create_engine(
    task_paths: Vec<PathBuf>,
    flows_dir: PathBuf,
    resources_dir: PathBuf,
    user_options: HashMap<String, serde_yaml::Value>,
    input: Arc<dyn InputBackend>,
    hwnd: isize,
) -> EngineHandle {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<EngineCommand>(16);
    let (event_tx, event_rx) = mpsc::channel::<EngineEvent>(64);
    let (cancel_tx, cancel_rx) = watch::channel(false);

    tokio::spawn(async move {
        // Wait for Start command
        match cmd_rx.recv().await {
            Some(EngineCommand::Start) => {}
            _ => return,
        }

        // Spawn cancel listener
        let cancel_tx_arc = Arc::new(cancel_tx);
        let cancel_tx_clone = cancel_tx_arc.clone();
        tokio::spawn(async move {
            if let Some(EngineCommand::Cancel) = cmd_rx.recv().await {
                let _ = cancel_tx_clone.send(true);
            }
        });

        for task_path in &task_paths {
            if *cancel_rx.borrow() {
                break;
            }

            // Derive flow_path from flows_dir using task file stem
            let task_stem = task_path.file_stem().unwrap_or_default();
            let flow_path = flows_dir.join(task_stem).with_extension("flow.yaml");
            let shared_dir = flows_dir.join("shared");
            let shared_dir_ref = if shared_dir.exists() { Some(shared_dir.as_path()) } else { None };

            let loaded: ExecTask = match loader::load_task(
                task_path,
                &flow_path,
                shared_dir_ref,
                &user_options,
            ) {
                Ok(t) => ExecTask {
                    name: t.name,
                    label: t.label,
                    entry: t.entry,
                    nodes: t.nodes,
                    disabled_nodes: t.disabled_nodes,
                },
                Err(e) => {
                    let name = task_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let _ = event_tx.send(EngineEvent::TaskCompleted(TaskResult {
                        task_name: name,
                        state: TaskState::Failed,
                        message: Some(format!("load error: {e}")),
                        duration_ms: 0,
                    })).await;
                    continue;
                }
            };

            let task_name = loaded.name.clone();
            let vision = match WgcVisionProvider::new(hwnd, resources_dir.clone()) {
                Ok(v) => v,
                Err(e) => {
                    let _ = event_tx.send(EngineEvent::TaskCompleted(TaskResult {
                        task_name,
                        state: TaskState::Failed,
                        message: Some(format!("vision init error: {e}")),
                        duration_ms: 0,
                    })).await;
                    continue;
                }
            };

            let executor_event_tx = event_tx.clone();
            let mut executor = Executor::new(
                loaded,
                Arc::new(vision),
                input.clone(),
                cancel_rx.clone(),
                ExecutorConfig::default(),
                executor_event_tx,
            );

            let _ = event_tx.send(EngineEvent::TaskStarted(task_name.clone())).await;

            let start = Instant::now();
            let executor_status = executor.run().await;
            let duration_ms = start.elapsed().as_millis() as u64;

            // Dump trace — emit as Tauri event AND write to log file
            let trace_jsonl = executor.dump_trace();
            if !trace_jsonl.is_empty() {
                let line_count = trace_jsonl.lines().count();
                info!(task = %task_name, trace_lines = line_count, "trace dump");
                // Write each trace line to the structured log file for AI analysis
                for line in trace_jsonl.lines() {
                    info!(target: "squire_engine::trace", task = %task_name, "{}", line);
                }
                let _ = event_tx.send(EngineEvent::TraceDump(trace_jsonl)).await;
            }

            let (state, message) = match executor_status {
                ExecutorStatus::Success => (TaskState::Success, None),
                ExecutorStatus::Cancelled => (TaskState::Cancelled, None),
                ExecutorStatus::Failed(msg) => (TaskState::Failed, Some(msg)),
                ExecutorStatus::Running => (TaskState::Failed, Some("terminated while running".into())),
            };

            let _ = event_tx.send(EngineEvent::TaskCompleted(TaskResult {
                task_name,
                state,
                message,
                duration_ms,
            })).await;
        }
    });

    EngineHandle { cmd_tx, event_rx }
}
