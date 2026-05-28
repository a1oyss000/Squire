<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# squire-vision

## Purpose
Computer vision subsystem. Handles window capture, template matching (via OpenCV FFI), and OCR (via Tesseract FFI and Windows Media OCR).

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Dependencies: image, windows crate (Win32 + WinRT OCR) |
| `src/lib.rs` | Module exports and DLL dependency checker |
| `src/capture.rs` | Window capture via Win32 GDI (BitBlt) and window enumeration |
| `src/matcher.rs` | Template matching using OpenCV FFI |
| `src/ocr.rs` | OCR via Tesseract FFI and Windows native OCR |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/ffi/` | FFI bindings to native libraries (see `src/ffi/AGENTS.md`) |
| `benches/` | Criterion benchmarks for NCC matching |

## For AI Agents

### Working In This Directory
- Feature flag `ffi` (default on) enables OpenCV/Tesseract bindings
- DLLs required at runtime: `opencv_world4100.dll`, `tesseract53.dll`
- `check_dependencies()` verifies DLLs are loadable before use
- Image type is `capture::Image { width, height, data: Arc<Vec<u8>> }` (BGRA32)

### Testing Requirements
- `cargo test -p squire-vision` — unit tests
- `cargo bench -p squire-vision` — NCC benchmark
- FFI tests require DLLs in PATH

### Common Patterns
- All platform-specific code gated with `#[cfg(windows)]` / `#[cfg(not(windows))]`
- Non-Windows stubs return `SquireError::Vision("Not supported")`
- Image data is BGRA 32-bit, top-down layout

## Dependencies

### Internal
- `squire-error` — error types

### External
- `image` 0.25 — image loading/decoding for templates
- `windows` 0.58 — Win32 GDI capture + WinRT OCR APIs
- OpenCV 4.10 (runtime DLL) — template matching
- Tesseract 5.3 (runtime DLL) — OCR engine

<!-- MANUAL: -->
