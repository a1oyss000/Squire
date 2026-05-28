<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# squire-input

## Purpose
Input simulation backends. Provides a trait-based abstraction over platform-specific input methods (Win32 API, ADB for Android emulators).

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Dependencies: windows crate (keyboard/mouse APIs) |
| `src/lib.rs` | `InputBackend` trait definition and `Point` struct |
| `src/winapi.rs` | Win32 `SendInput` implementation |
| `src/adb.rs` | ADB-based input for Android emulators |

## For AI Agents

### Working In This Directory
- Implement `InputBackend` trait for new input methods
- Trait methods: `click`, `double_click`, `drag`, `key_press`
- `Point { x: i32, y: i32 }` uses screen coordinates

### Testing Requirements
- Input tests are inherently side-effectful — test on a dedicated window
- `cargo build -p squire-input` to verify compilation

### Common Patterns
- Backend implementations are `Send + Sync` for use across async tasks
- Win32 backend uses `SendInput` with `INPUT_MOUSE` / `INPUT_KEYBOARD`

## Dependencies

### Internal
- `squire-error` — error types

### External
- `windows` 0.58 — `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_WindowsAndMessaging`

<!-- MANUAL: -->
