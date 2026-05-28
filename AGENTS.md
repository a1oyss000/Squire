<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# Squire

## Purpose
A Windows game automation tool built with Tauri. Executes YAML-defined task sequences against a target window using computer vision (template matching, OCR) and simulated input (clicks, drags, keypresses).

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace manifest defining all crates and shared dependencies |
| `tauri.conf.json` | Tauri application configuration (lives in `src-tauri/`) |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `crates/` | Rust library crates (see `crates/AGENTS.md`) |
| `src-tauri/` | Tauri application shell and IPC commands (see `src-tauri/AGENTS.md`) |
| `src-ui/` | React frontend for task management UI (see `src-ui/AGENTS.md`) |
| `tasks/` | YAML task definitions loaded at runtime (see `tasks/AGENTS.md`) |
| `templates/` | Image templates for vision matching (see `templates/AGENTS.md`) |
| `docs/` | Project documentation (see `docs/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- This is a Cargo workspace — run `cargo build` from root to build all crates
- The Tauri app (`src-tauri`) depends on all workspace crates
- Windows-only features are gated behind `cfg(windows)` — non-Windows builds compile but stub out vision/input

### Testing Requirements
- `cargo test` from root runs all crate tests
- `cargo clippy` for lint checks
- Vision/input tests require a Windows environment with display access

### Common Patterns
- Error types centralized in `squire-error`, re-exported as `squire_error::Result<T>`
- Async runtime is Tokio; engine uses channels (`mpsc`, `watch`) for control flow
- Task definitions are YAML files deserialized via serde
- Template paths in YAML are resolved relative to the project base directory at runtime

## Dependencies

### External
- Tauri 2.x — desktop app framework
- Tokio — async runtime
- OpenCV 4.10 (DLL) — template matching via FFI
- Tesseract 5.3 (DLL) — OCR via FFI
- Windows crate 0.58 — Win32 API bindings

<!-- MANUAL: -->
