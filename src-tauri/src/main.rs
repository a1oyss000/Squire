#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use squire_engine::config::{self, TaskDefinition};
use squire_engine::runner::{EngineCommand, EngineEvent};
use squire_engine::scheduler;
use squire_engine::state::TaskResult;
use squire_input::winapi::WinApiBackend;
use squire_vision::capture;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use tokio::sync::mpsc;

struct AppState {
    tasks_dir: PathBuf,
    results: Mutex<Vec<TaskResult>>,
    engine_cmd: Mutex<Option<mpsc::Sender<EngineCommand>>>,
}

#[tauri::command]
fn get_tasks(state: State<AppState>) -> Result<Vec<String>, String> {
    let dir = &state.tasks_dir;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut tasks = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            if let Some(name) = path.file_stem() {
                tasks.push(name.to_string_lossy().to_string());
            }
        }
    }
    Ok(tasks)
}

#[tauri::command]
fn get_task_config(state: State<AppState>, name: String) -> Result<String, String> {
    let path = safe_task_path(&state.tasks_dir, &name)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn safe_task_path(tasks_dir: &std::path::Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Invalid task name".to_string());
    }
    let path = tasks_dir.join(format!("{}.yaml", name));
    Ok(path)
}

// PLACEHOLDER_COMMANDS

#[tauri::command]
async fn start_execution(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    window_title: String,
    task_names: Vec<String>,
) -> Result<(), String> {
    let hwnd = capture::find_window(&window_title).map_err(|e| e.to_string())?;

    let dir = &state.tasks_dir;
    let mut tasks: Vec<TaskDefinition> = Vec::new();
    for name in &task_names {
        let path = safe_task_path(dir, name)?;
        match config::load_task(&path) {
            Ok(task) => tasks.push(task),
            Err(e) => {
                tracing::warn!("Failed to load {}: {}", name, e);
                return Err(format!("Failed to load task '{}': {}", name, e));
            }
        }
    }

    let input = Arc::new(WinApiBackend::new());
    let handle = scheduler::create_engine(tasks, input, hwnd);

    *state.engine_cmd.lock().unwrap() = Some(handle.cmd_tx.clone());
    handle.cmd_tx.send(EngineCommand::Start).await.map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut event_rx = handle.event_rx;
        while let Some(event) = event_rx.recv().await {
            match &event {
                EngineEvent::TaskStarted(name) => {
                    let _ = app_clone.emit("engine-event", format!("started:{}", name));
                }
                EngineEvent::StepCompleted { task, step_index } => {
                    let _ = app_clone.emit("engine-event", format!("step_ok:{}:{}", task, step_index));
                }
                EngineEvent::StepFailed { task, step_index, error } => {
                    let _ = app_clone.emit("engine-event", format!("step_fail:{}:{}:{}", task, step_index, error));
                }
                EngineEvent::TaskCompleted(result) => {
                    let _ = app_clone.emit("engine-event", format!("completed:{}:{:?}", result.task_name, result.state));
                }
                _ => {}
            }
        }
        let _ = app_clone.emit("engine-event", "done".to_string());
    });

    Ok(())
}

#[tauri::command]
async fn stop_execution(state: State<'_, AppState>) -> Result<(), String> {
    let cmd_tx = state.engine_cmd.lock().unwrap().take();
    if let Some(tx) = cmd_tx {
        tx.send(EngineCommand::Cancel).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("squire=debug,info")
        .init();

    tracing::info!("Starting Squire");

    tauri::Builder::default()
        .manage(AppState {
            tasks_dir: PathBuf::from("tasks"),
            results: Mutex::new(Vec::new()),
            engine_cmd: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            get_task_config,
            start_execution,
            stop_execution,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

