use crate::config::TaskDefinition;
use crate::runner::{self, EngineCommand, EngineEvent, EngineHandle, RunContext};
use crate::state::{TaskResult, TaskState};
use squire_input::InputBackend;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

pub async fn run_queue(tasks: Vec<TaskDefinition>) -> Vec<TaskResult> {
    let mut results = Vec::new();

    for task in &tasks {
        if !task.enabled {
            tracing::info!("Skipping disabled task: {}", task.name);
            continue;
        }
        let result = runner::run_task(task).await;
        tracing::info!("Task '{}' completed: {:?}", task.name, result.state);
        results.push(result);
    }

    results
}

pub fn create_engine(
    tasks: Vec<TaskDefinition>,
    input: Arc<dyn InputBackend>,
    window_hwnd: isize,
) -> EngineHandle {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<EngineCommand>(16);
    let (event_tx, event_rx) = mpsc::channel::<EngineEvent>(64);
    let (cancel_tx, cancel_rx) = watch::channel(false);

    tokio::spawn(async move {
        if let Some(EngineCommand::Start) = cmd_rx.recv().await {
            let ctx = RunContext {
                input,
                window_hwnd,
                cancel_rx,
            };

            let cancel_tx_clone = cancel_tx.clone();
            tokio::spawn(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    if matches!(cmd, EngineCommand::Cancel) {
                        let _ = cancel_tx_clone.send(true);
                        break;
                    }
                }
            });

            for task in &tasks {
                if *ctx.cancel_rx.borrow() {
                    break;
                }
                if !task.enabled {
                    tracing::info!("Skipping disabled task: {}", task.name);
                    continue;
                }
                let result = runner::run_task_with_context(task, &ctx, Some(&event_tx)).await;
                let _ = event_tx.send(EngineEvent::TaskCompleted(result.clone())).await;

                if result.state == TaskState::Failed {
                    tracing::warn!("Task '{}' failed, stopping queue", task.name);
                    break;
                }
            }
        }
    });

    EngineHandle { cmd_tx, event_rx }
}

