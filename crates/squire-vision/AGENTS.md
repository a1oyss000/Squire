<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# squire-vision

## 概述
计算机视觉子系统。处理窗口截图、模板匹配（通过 OpenCV FFI）和 OCR（通过 Tesseract FFI 及 Windows Media OCR）。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | 依赖：image、windows crate（Win32 + WinRT OCR） |
| `src/lib.rs` | 模块导出和 DLL 依赖检查器 |
| `src/capture.rs` | 窗口截图：WGC（主）+ BitBlt（备用），自动裁剪到客户区 |
| `src/matcher.rs` | 使用 OpenCV FFI 的模板匹配 |
| `src/ocr.rs` | 通过 Tesseract FFI 和 Windows 原生 OCR 进行文字识别 |

## 子目录

| 目录 | 用途 |
|------|------|
| `src/ffi/` | 原生库的 FFI 绑定（见 `src/ffi/AGENTS.md`） |
| `benches/` | NCC 匹配的 Criterion 基准测试 |

## AI Agent 指南

### 在此目录工作
- Feature flag `ffi`（默认开启）启用 OpenCV/Tesseract 绑定
- 运行时需要 DLL：`opencv_world4100.dll`、`tesseract53.dll`
- `check_dependencies()` 在使用前验证 DLL 可加载
- 图像类型为 `capture::Image { width, height, data: Arc<Vec<u8>> }`（BGRA32）

### 测试要求
- `cargo test -p squire-vision` — 单元测试
- `cargo bench -p squire-vision` — NCC 基准测试
- FFI 测试需要 DLL 在 PATH 中

### 常见模式
- 所有平台特定代码通过 `#[cfg(windows)]` / `#[cfg(not(windows))]` 门控
- 非 Windows stub 返回 `SquireError::Vision("Not supported")`
- 图像数据为 BGRA 32 位，自上而下布局

## 依赖

### 内部
- `squire-error` — 错误类型

### 外部
- `image` 0.25 — 模板图片加载/解码
- `windows` 0.58 — Win32 GDI 截图 + WinRT OCR API
- OpenCV 4.10（运行时 DLL）— 模板匹配
- Tesseract 5.3（运行时 DLL）— OCR 引擎

<!-- MANUAL: -->
