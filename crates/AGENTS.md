<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# crates

## Purpose
Container for the Rust library crates that form the core automation logic. Each crate has a single responsibility and is consumed by the Tauri app.

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `squire-error/` | Shared error enum and Result type (see `squire-error/AGENTS.md`) |
| `squire-vision/` | Screen capture, template matching, OCR (see `squire-vision/AGENTS.md`) |
| `squire-input/` | Input simulation backends (see `squire-input/AGENTS.md`) |
| `squire-engine/` | Task execution engine — config, runner, scheduler (see `squire-engine/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Each crate is independent; add new crates here and register them in the root `Cargo.toml` workspace members
- Dependency graph: `squire-error` ← `squire-vision`, `squire-input` ← `squire-engine`

<!-- MANUAL: -->
