use crate::config::{Action, FailStrategy, StepDef, StepTarget, SuccessMarker, TaskDefinition};
use crate::state::{TaskResult, TaskState};
use squire_error::Result;
use squire_input::{InputBackend, Point};
use squire_vision::capture;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    Start,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    TaskStarted(String),
    StepStarted { task: String, step_index: usize },
    StepCompleted { task: String, step_index: usize },
    StepFailed { task: String, step_index: usize, error: String },
    TaskCompleted(TaskResult),
}

pub struct EngineHandle {
    pub cmd_tx: mpsc::Sender<EngineCommand>,
    pub event_rx: mpsc::Receiver<EngineEvent>,
}

pub struct RunContext {
    pub input: Arc<dyn InputBackend>,
    pub window_hwnd: isize,
    pub cancel_rx: watch::Receiver<bool>,
}

pub async fn run_task_with_context(
    task: &TaskDefinition,
    ctx: &RunContext,
    event_tx: Option<&mpsc::Sender<EngineEvent>>,
) -> TaskResult {
    let start = Instant::now();

    if let Some(tx) = event_tx {
        let _ = tx.send(EngineEvent::TaskStarted(task.name.clone())).await;
    }

    for (i, step) in task.steps.iter().enumerate() {
        if *ctx.cancel_rx.borrow() {
            return TaskResult {
                task_name: task.name.clone(),
                state: TaskState::Cancelled,
                message: Some("Cancelled by user".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        if let Some(tx) = event_tx {
            let _ = tx.send(EngineEvent::StepStarted {
                task: task.name.clone(),
                step_index: i,
            }).await;
        }

        let result = execute_step(step, ctx).await;
        match result {
            Ok(()) => {
                if let Some(tx) = event_tx {
                    let _ = tx.send(EngineEvent::StepCompleted {
                        task: task.name.clone(),
                        step_index: i,
                    }).await;
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if let Some(tx) = event_tx {
                    let _ = tx.send(EngineEvent::StepFailed {
                        task: task.name.clone(),
                        step_index: i,
                        error: err_msg.clone(),
                    }).await;
                }

                match step.on_fail {
                    FailStrategy::Skip => continue,
                    FailStrategy::Pause => {
                        return TaskResult {
                            task_name: task.name.clone(),
                            state: TaskState::Failed,
                            message: Some(format!("Paused at step {}: {}", i + 1, err_msg)),
                            duration_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                    FailStrategy::Retry => {
                        let max = task.config.retry_count;
                        let mut success = false;
                        for attempt in 0..max {
                            sleep(Duration::from_millis(500)).await;
                            if *ctx.cancel_rx.borrow() {
                                return TaskResult {
                                    task_name: task.name.clone(),
                                    state: TaskState::Cancelled,
                                    message: Some("Cancelled during retry".to_string()),
                                    duration_ms: start.elapsed().as_millis() as u64,
                                };
                            }
                            tracing::info!("Retry {}/{} for step {}", attempt + 1, max, i + 1);
                            if execute_step(step, ctx).await.is_ok() {
                                success = true;
                                break;
                            }
                        }
                        if !success {
                            return TaskResult {
                                task_name: task.name.clone(),
                                state: TaskState::Failed,
                                message: Some(format!("Step {} failed after {} retries: {}", i + 1, max, err_msg)),
                                duration_ms: start.elapsed().as_millis() as u64,
                            };
                        }
                    }
                }
            }
        }
    }

    match &task.success_marker {
        Some(SuccessMarker::Template { path, threshold }) => {
            match capture::capture_window(ctx.window_hwnd) {
                Ok(image) => {
                    let thr = threshold.unwrap_or(0.8);
                    if squire_vision::matcher::match_template(&image, path, thr).is_err() {
                        return TaskResult {
                            task_name: task.name.clone(),
                            state: TaskState::Failed,
                            message: Some("Success marker template not matched".to_string()),
                            duration_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
                Err(e) => {
                    return TaskResult {
                        task_name: task.name.clone(),
                        state: TaskState::Failed,
                        message: Some(format!("Failed to capture window for success marker: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            }
        }
        Some(SuccessMarker::Ocr { text, region }) => {
            match capture::capture_window(ctx.window_hwnd) {
                Ok(image) => {
                    if squire_vision::ocr::find_text(&image, text, region.as_ref()).is_err() {
                        return TaskResult {
                            task_name: task.name.clone(),
                            state: TaskState::Failed,
                            message: Some(format!("Success marker text not found: {}", text)),
                            duration_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
                Err(e) => {
                    return TaskResult {
                        task_name: task.name.clone(),
                        state: TaskState::Failed,
                        message: Some(format!("Failed to capture window for success marker: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
            }
        }
        None => {}
    }

    TaskResult {
        task_name: task.name.clone(),
        state: TaskState::Success,
        message: None,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn execute_step(step: &StepDef, ctx: &RunContext) -> Result<()> {
    match &step.action {
        Action::Wait { ms } => {
            sleep(Duration::from_millis(*ms)).await;
            return Ok(());
        }
        _ => {}
    }

    let point = resolve_target(&step.target, ctx)?;

    match &step.action {
        Action::Click => {
            ctx.input.click(point)?;
        }
        Action::Swipe { to_x, to_y } => {
            ctx.input.drag(point, Point { x: *to_x, y: *to_y })?;
        }
        Action::Wait { .. } => unreachable!(),
    }

    Ok(())
}

fn resolve_target(target: &StepTarget, ctx: &RunContext) -> Result<Point> {
    match target {
        StepTarget::Coordinate { x, y } => Ok(Point { x: *x, y: *y }),
        StepTarget::Template { path, threshold } => {
            let image = capture::capture_window(ctx.window_hwnd)?;
            let thr = threshold.unwrap_or(0.8);
            let result = squire_vision::matcher::match_template(&image, path, thr)?;
            let screen_pt = client_to_screen(ctx.window_hwnd, result.center.x, result.center.y);
            Ok(screen_pt)
        }
        StepTarget::Ocr { text, region } => {
            let image = capture::capture_window(ctx.window_hwnd)?;
            let found = squire_vision::ocr::find_text(&image, text, region.as_ref())?;
            let screen_pt = client_to_screen(ctx.window_hwnd, found.x, found.y);
            Ok(screen_pt)
        }
    }
}

#[cfg(windows)]
fn client_to_screen(hwnd: isize, x: i32, y: i32) -> Point {
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::Graphics::Gdi::ClientToScreen as WinClientToScreen;

    let hwnd_val = HWND(hwnd as *mut _);
    let mut pt = POINT { x, y };
    unsafe { let _ = WinClientToScreen(hwnd_val, &mut pt); }
    Point { x: pt.x, y: pt.y }
}

#[cfg(not(windows))]
fn client_to_screen(_hwnd: isize, x: i32, y: i32) -> Point {
    Point { x, y }
}

