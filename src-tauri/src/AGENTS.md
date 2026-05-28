<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# src (Tauri)

## Purpose
Rust source for the Tauri application. Contains the entry point, IPC command handlers, and application state management.

## Key Files

| File | Description |
|------|-------------|
| `main.rs` | App entry: Tauri setup, logging init, command registration, engine lifecycle |

## For AI Agents

### Working In This Directory
- All IPC commands are `#[tauri::command]` functions registered in `generate_handler![]`
- `AppState` holds: tasks directory path, execution results, engine command sender
- Logging: rolling daily files via `tracing-appender`, JSON format for file, compact for console
- Tasks dir resolves to project root `tasks/` in dev, resource dir in production

### Common Patterns
- Commands return `Result<T, String>` for Tauri IPC serialization
- `safe_task_path()` validates task names to prevent path traversal
- Engine start: load YAML → create engine → spawn event forwarding task
- Engine stop: send `EngineCommand::Cancel` via stored channel sender

<!-- MANUAL: -->
