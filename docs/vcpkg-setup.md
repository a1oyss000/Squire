# Native Dependencies Setup (vcpkg)

Squire uses OpenCV and Tesseract via FFI for template matching and OCR.
The `ffi` feature (enabled by default) requires these native libraries at build time.

## Quick Start

```powershell
# Install vcpkg (if not already installed)
git clone https://github.com/microsoft/vcpkg C:\vcpkg
C:\vcpkg\bootstrap-vcpkg.bat

# Install dependencies
C:\vcpkg\vcpkg install opencv4:x64-windows tesseract:x64-windows

# Set environment variable
$env:VCPKG_ROOT = "C:\vcpkg"
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `VCPKG_ROOT` | Path to vcpkg installation (auto-discovers libs) |
| `OPENCV_DIR` | Override: direct path to OpenCV install directory |
| `TESSERACT_DIR` | Override: direct path to Tesseract install directory |

The build script checks in order: env var override, `VCPKG_ROOT`, `C:\vcpkg` default.

## Runtime DLLs

The following DLLs must be in PATH or alongside the executable:

- `opencv_world4100.dll` (or individual `opencv_core4`, `opencv_imgproc4`, `opencv_imgcodecs4`)
- `tesseract53.dll`
- `leptonica-1.84.1.dll`

## Building Without Native Libs

To build without FFI (uses WinRT OCR and pure-Rust NCC fallback):

```powershell
cargo build -p squire-vision --no-default-features
```

## Tessdata

Tesseract requires language data files. Download from:
https://github.com/tesseract-ocr/tessdata_fast

Place in a `tessdata/` directory and pass the path to `OcrEngine::new()`.
