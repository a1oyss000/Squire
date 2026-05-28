<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# ffi

## Purpose
Foreign Function Interface bindings to native C libraries used for computer vision. Provides safe Rust wrappers around raw C APIs.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Module re-exports |
| `opencv.rs` | OpenCV C API bindings — CvMat, template matching, color conversion |
| `tesseract.rs` | Tesseract C API bindings — OCR engine initialization and text extraction |

## For AI Agents

### Working In This Directory
- These are raw `extern "C"` declarations — match signatures exactly to the DLL exports
- `OwnedCvMat` and `HeaderCvMat` provide RAII wrappers for OpenCV matrix pointers
- OpenCV uses the legacy C API (`cv*` functions), not the C++ API
- Tesseract bindings target the C API (`TessBaseAPI*` functions)

### Testing Requirements
- Requires DLLs in PATH: `opencv_world4100.dll`, `tesseract53.dll`
- Test via higher-level `matcher.rs` and `ocr.rs` which consume these bindings

### Common Patterns
- All pointers checked for null before wrapping in `Option<Self>`
- RAII Drop impls call the corresponding release/destroy functions
- Constants match OpenCV/Tesseract C header values exactly

## Dependencies

### External
- OpenCV 4.10 C API (runtime DLL)
- Tesseract 5.3 C API (runtime DLL)

<!-- MANUAL: -->
