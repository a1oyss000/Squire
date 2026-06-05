<!-- OMC:START -->
<!-- OMC:VERSION:4.14.4 -->

# Squire — Game Automation Engine

A Tauri desktop app that automates game tasks using computer vision (WGC screen capture, OpenCV template matching, Tesseract OCR) and simulated input (Win32 API / ADB).

## Architecture

```
src-ui/             React (TypeScript, Zustand, Tailwind) — task list, window picker, execution log
src-tauri/          Tauri v2 (Rust) — command handlers, logging, admin escalation
crates/
  squire-engine/    Core engine — YAML schema, loader, executor, scheduler
  squire-vision/    CV layer — WGC capture, NCC template matching, Tesseract OCR
  squire-input/     Input backends — Win32 mouse/keyboard, ADB
  squire-error/     Shared error types
```

## Engine crate (`squire-engine/src/`)

| Module | Role |
|--------|------|
| `schema/` | YAML data models: `TaskDocument`, `FlowDocument`, `NodeDef`, `RecognizeCondition`, `Action` |
| `loader.rs` | Parse task.yaml + flow.yaml, resolve `$variables`, merge shared `.flow.yaml` templates, compute `disabled_nodes` from bindings |
| `executor.rs` | Runtime: call-stack based graph walker, vision matching loop, action dispatch, timeout/max-hit skip logic |
| `recognize.rs` | `VisionProvider` trait + `evaluate()` — template, OCR, color, AND/OR composite conditions |
| `action.rs` | Execute primitive actions: click, click_at, click_offset, click_repeat, custom |
| `scheduler.rs` | `EngineHandle` — channel-based command/event interface, one tokio task per task file |
| `state.rs` | `TaskState` enum (Pending/Running/Success/Failed/Skipped/Cancelled), `TaskResult` |
| `trace.rs` | Capped ring buffer (10K entries), JSONL dump |
| `vision_impl.rs` | `WgcVisionProvider` — adapts `squire-vision` into the engine's `VisionProvider` trait |

### Data flow

```
resources/
  tasks/<name>.yaml ──┐
                       ├──► loader ──► LoadedTask ──► Executor ──► TaskResult
  flows/<name>.flow.yaml─┘               │               │
                                          │               ▼
                                  $var substitution    TraceLog
                                  binding→disabled     EngineEvents → UI
```

### Directory layout

```
resources/
  tasks/                  Task YAML + template images
    <name>.yaml           Task metadata (name, label, flow, options, bindings)
    templates/            PNG templates referenced in flow recognize conditions
  flows/                  Flow YAML + shared includes
    <name>.flow.yaml      Node graph (entry + nodes)
    shared/               Reusable .flow.yaml snippets ($variable substitution)
```

### Task format (two files)

**resources/tasks/<name>.yaml** — metadata, options, resources, bindings:
```yaml
name: my-task
label: My Task
flow: my-flow
group: [optional]
options:
  difficulty:
    type: select
    default: easy
    cases: [easy, hard]
resources:
  easy:
    threshold: "0.8"
  hard:
    threshold: "0.95"
bindings:
  - nodes: [bonus-node]
    enabled_by: enable_bonus
```

**resources/flows/<name>.flow.yaml** — state machine graph:
```yaml
entry: start-node
nodes:
  start-node:
    action: click
    next: [next-node, end-task]
  next-node:
    recognize:
      template: resources/tasks/templates/button.png
    action:
      click: [100, 200]
    pre_wait: { freeze: 500 }
    post_wait: 1000
    max_hit: 3
    timeout: 30000
    next: [end-task]
```

**Shared templates** in `resources/flows/shared/*.flow.yaml` are referenced via `includes: [{use: template_name, with: {...}}]`.

### Key recognize conditions
- `template` — NCC template matching with threshold
- `ocr` — Tesseract text recognition with replacements
- `color` — pixel range counting in ROI
- `and` / `or` — composite conditions referencing other nodes

## Build & Test

```bash
cargo build                    # full workspace
cargo test -p squire-engine    # engine unit tests
cargo clippy                   # lint
```

## UI

React + Zustand + Tailwind. Components: `TaskList`, `TaskConfig`, `WindowPicker`, `ExecutionStatus`.
Tauri commands: `get_tasks`, `get_task_config`, `get_task_options`, `list_windows`, `start_execution`, `stop_execution`.

<!-- OMC:END -->
