<p align="center">
  <img src="src-ui/src/assets/hero.png" alt="Squire" width="120" />
</p>

<h1 align="center">Squire</h1>

<p align="center">
  Game automation through computer vision. Write YAML, watch it work.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows-0078D4?style=flat&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?style=flat&logo=tauri&logoColor=black" alt="Tauri" />
  <img src="https://img.shields.io/badge/Rust-2021-000000?style=flat&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react&logoColor=black" alt="React" />
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License" />
</p>

---

Squire automates repetitive game tasks by capturing a target window, locating UI elements via template matching or OCR, and simulating input. Define your automation as a YAML task file — no coding required.

## Features

- **Template Matching** — Locate buttons and UI elements using OpenCV-powered image matching
- **OCR** — Find text on screen via Tesseract or Windows native OCR
- **Input Simulation** — Click, drag, and keypress via Win32 API or ADB (Android emulators)
- **YAML Task Definitions** — Declarative automation with steps, targets, and failure strategies
- **Smart Failure Handling** — Per-step retry, skip, or pause strategies with configurable retry counts
- **Success Markers** — Verify task completion via template or OCR checks
- **Desktop UI** — Dark-themed interface with window picker, drag-and-drop task ordering, and live execution logs

## Quick Start

1. Download the latest release or build from source
2. Place OpenCV and Tesseract DLLs in your PATH (see [Dependencies](#dependencies))
3. Launch Squire, pick a target window, enable your tasks, and click Start

## Task Definition

Tasks are YAML files that describe an automation sequence:

```yaml
name: NIKKE Daily Mail
enabled: true
description: Collect daily mail rewards in NIKKE

steps:
  - action:
      type: click
    target:
      type: template
      path: templates/nikke/mail_button.png
      threshold: 0.8
    timeout_ms: 10000
    on_fail: retry

  - action:
      type: wait
      ms: 1500
    target:
      type: coordinate
      x: 0
      y: 0
    timeout_ms: 2000

  - action:
      type: click
    target:
      type: template
      path: templates/nikke/collect_all.png
      threshold: 0.8
    timeout_ms: 10000
    on_fail: skip

config:
  fail_strategy: retry
  retry_count: 3

success_marker:
  type: ocr
  text: "领取成功"
  region: [300, 400, 400, 100]
```

### Actions

| Type | Description |
|------|-------------|
| `click` | Click at the resolved target position |
| `wait` | Pause execution for a duration |
| `swipe` | Drag from target position to a destination |

### Targets

| Type | Description |
|------|-------------|
| `template` | Match an image template on screen (confidence threshold) |
| `ocr` | Find text on screen, optionally within a region |
| `coordinate` | Fixed screen coordinates |

### Failure Strategies

| Strategy | Behavior |
|----------|----------|
| `retry` | Retry the step up to `retry_count` times (default) |
| `skip` | Skip the failed step and continue |
| `pause` | Stop execution and report failure |

## Architecture

```
┌─────────────────────────────────────────────────┐
│  src-ui (React 19 + Tailwind + Zustand)         │
├─────────────────────────────────────────────────┤
│  src-tauri (Tauri 2 IPC layer)                  │
├──────────┬──────────┬──────────┬────────────────┤
│  engine  │  vision  │  input   │  error         │
│  --------│----------│----------│  ──────         │
│  config  │  capture │  winapi  │  shared types  │
│  runner  │  matcher │  adb     │                │
│  sched.  │  ocr     │          │                │
│          │  ffi/    │          │                │
└──────────┴──────────┴──────────┴────────────────┘
```

| Crate | Role |
|-------|------|
| `squire-error` | Shared error types across all crates |
| `squire-vision` | Window capture, template matching (OpenCV), OCR (Tesseract) |
| `squire-input` | Input simulation via Win32 API and ADB |
| `squire-engine` | Task loading, step execution, scheduling, retry logic |

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| OpenCV | 4.10 | Template matching (`opencv_world4100.dll`) |
| Tesseract | 5.3 | OCR engine (`tesseract53.dll`) |
| Rust | 2021 edition | Backend language |
| Node.js | 18+ | Frontend build tooling |

Place DLLs alongside the executable or ensure they are in your system PATH.

## Building from Source

```bash
# Prerequisites: Rust toolchain, Node.js 18+, vcpkg (for OpenCV/Tesseract)

# Install frontend dependencies
cd src-ui && npm install && cd ..

# Build the app (dev mode)
cargo tauri dev

# Build for production
cargo tauri build
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo clippy` and `cargo test`
5. Submit a pull request

## License

[MIT](LICENSE)
