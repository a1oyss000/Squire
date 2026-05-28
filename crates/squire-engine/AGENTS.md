<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# squire-engine

## Purpose
Task execution engine. Loads YAML task definitions, schedules execution against a target window, runs steps with retry/skip/pause strategies, and reports events.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Dependencies: all sibling crates + tokio + serde_yaml |
| `src/lib.rs` | Module exports: config, runner, scheduler, state |
| `src/config.rs` | Task/step/action data model and YAML deserialization |
| `src/runner.rs` | Step execution logic, target resolution, retry loops |
| `src/scheduler.rs` | Engine lifecycle — spawns async task runner with channels |
| `src/state.rs` | `TaskResult` and `TaskState` types |

## For AI Agents

### Working In This Directory
- `config.rs` defines the YAML schema — changes here affect task file format
- `runner.rs` is the hot path: capture → match/OCR → input → report
- `scheduler.rs` manages the engine lifecycle via `mpsc`/`watch` channels
- Engine communicates via `EngineCommand` (Start/Cancel) and `EngineEvent`

### Testing Requirements
- `cargo test -p squire-engine`
- Config parsing tests can run anywhere; runner tests need Windows + DLLs

### Common Patterns
- `RunContext` bundles input backend, window handle, cancel signal, and base dir
- Steps resolve targets to `Point` then dispatch to `InputBackend`
- Template paths resolved via `ctx.base_dir.join(path)` at runtime
- Failure handling: per-step `on_fail` overrides task-level `fail_strategy`

## Dependencies

### Internal
- `squire-error` — error types
- `squire-vision` — capture + matching + OCR
- `squire-input` — input simulation

### External
- `tokio` + `tokio-util` — async runtime and channels
- `serde` + `serde_yaml` — YAML config deserialization
- `tracing` — structured logging

<!-- MANUAL: -->
