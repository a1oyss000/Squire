<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# src-tauri

## Purpose
Tauri 2 application shell. Defines IPC commands exposed to the frontend, manages application state, and wires together the engine crates.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | App dependencies — all workspace crates + Tauri |
| `src/main.rs` | Entry point: Tauri setup, command handlers, logging init |
| `tauri.conf.json` | Tauri config (window size, app name, capabilities) |
| `build.rs` | Tauri build script |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | Rust source for the Tauri app (see `src/AGENTS.md`) |
| `capabilities/` | Tauri capability permission files |
| `icons/` | Application icons |
| `gen/` | Auto-generated Tauri schemas (do not edit) |

## For AI Agents

### Working In This Directory
- Add new IPC commands in `src/main.rs` and register in `generate_handler![]`
- Use `State<AppState>` for shared state access in commands
- Logging goes to rolling daily files via `tracing-appender`

### Testing Requirements
- `cargo build -p squire-app` to verify compilation
- IPC commands are tested via the frontend or integration tests

<!-- MANUAL: -->
