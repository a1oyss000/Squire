#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use squire_engine::config::{self, TaskDefinition};
use squire_engine::runner::{EngineCommand, EngineEvent};
use squire_engine::scheduler;
use squire_engine::state::TaskResult;
use squire_input::winapi::WinApiBackend;
use squire_vision::capture::{self, WindowInfo};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tokio::sync::mpsc;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct AppState {
    tasks_dir: PathBuf,
    results: Mutex<Vec<TaskResult>>,
    engine_cmd: Mutex<Option<mpsc::Sender<EngineCommand>>>,
}

/// Holds the tracing worker guard so the non-blocking writer stays alive.
struct LogGuard(WorkerGuard);

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
fn list_windows() -> Vec<WindowInfo> {
    capture::list_windows().unwrap_or_default()
}

#[tauri::command]
async fn start_execution(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    hwnd: isize,
    task_names: Vec<String>,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::IsWindow;
        if !unsafe { IsWindow(HWND(hwnd as *mut _)) }.as_bool() {
            return Err("Window no longer exists (stale handle)".to_string());
        }
    }

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
    let base_dir = state.tasks_dir.clone();
    let handle = scheduler::create_engine(tasks, input, hwnd, base_dir);

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
    #[cfg(windows)]
    if !is_elevated() {
        relaunch_as_admin();
        return;
    }

    tauri::Builder::default()
        .setup(|app| {
            // Resolve logs directory using Tauri's app log dir
            let logs_dir = app.path().app_log_dir().expect("failed to resolve app log dir");
            std::fs::create_dir_all(&logs_dir).expect("failed to create logs directory");

            let file_appender = tracing_appender::rolling::daily(&logs_dir, "squire.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let console_layer = fmt::layer().compact();
            let file_layer = fmt::layer().json().with_writer(non_blocking);
            let filter = EnvFilter::new("squire=debug,info");

            tracing_subscriber::registry()
                .with(filter)
                .with(console_layer)
                .with(file_layer)
                .init();

            // Keep the guard alive for the app's lifetime
            app.manage(LogGuard(guard));

            tracing::info!("Starting Squire");
            tracing::info!("Logs directory: {}", logs_dir.display());

            // Resolve tasks directory:
            // In dev mode, use the project root's tasks/ directory.
            // In production, use a tasks/ dir next to the resource path.
            let tasks_dir = if cfg!(debug_assertions) {
                // CARGO_MANIFEST_DIR points to src-tauri/ at compile time
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                manifest_dir.parent().unwrap().join("tasks")
            } else {
                app.path().resource_dir()
                    .expect("failed to resolve resource dir")
                    .join("tasks")
            };

            tracing::info!("Tasks directory: {}", tasks_dir.display());

            app.manage(AppState {
                tasks_dir,
                results: Mutex::new(Vec::new()),
                engine_cmd: Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            get_task_config,
            list_windows,
            start_execution,
            stop_execution,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(windows)]
fn is_elevated() -> bool {
    use windows::Win32::Foundation::{HANDLE, CloseHandle};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        let _ = CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

#[cfg(windows)]
fn relaunch_as_admin() {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{PCWSTR, w};

    let exe = std::env::current_exe().unwrap_or_default();
    let exe_wide: Vec<u16> = exe.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}