use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, watch};

use crate::executor::{Executor, ExecutorConfig, ExecutorStatus, LoadedTask as ExecTask};
use crate::loader;
use crate::state::{TaskResult, TaskState};
use crate::vision_impl::WgcVisionProvider;
use squire_input::InputBackend;

pub enum EngineCommand {
    Start,
    Cancel,
}

pub enum EngineEvent {
    TaskStarted(String),
    NodeEntered { task: String, node: String },
    NodeCompleted { task: String, node: String },
    NodeSkipped { task: String, node: String, reason: String },
    TaskCompleted(TaskResult),
}

pub struct EngineHandle {
    pub cmd_tx: mpsc::Sender<EngineCommand>,
    pub event_rx: mpsc::Receiver<EngineEvent>,
}

pub fn create_engine(
    task_paths: Vec<PathBuf>,
    flows_dir: PathBuf,
    user_options: HashMap<String, serde_yaml::Value>,
    input: Arc<dyn InputBackend>,
    hwnd: isize,
) -> EngineHandle {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<EngineCommand>(16);
    let (event_tx, event_rx) = mpsc::channel::<EngineEvent>(64);
    let (cancel_tx, cancel_rx) = watch::channel(false);

    tokio::spawn(async move {
        // Wait for Start command
        loop {
            match cmd_rx.recv().await {
                Some(EngineCommand::Start) => break,
                Some(EngineCommand::Cancel) | None => return,
            }
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
            let vision = WgcVisionProvider::new(hwnd);

            let mut executor = Executor::new(
                loaded,
                Arc::new(vision),
                input.clone(),
                cancel_rx.clone(),
                ExecutorConfig::default(),
            );

            let _ = event_tx.send(EngineEvent::TaskStarted(task_name.clone())).await;

            let start = Instant::now();
            let executor_status = executor.run().await;
            let duration_ms = start.elapsed().as_millis() as u64;

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
