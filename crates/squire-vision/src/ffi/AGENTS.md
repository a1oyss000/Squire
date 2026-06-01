<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# ffi

## 概述
计算机视觉所用原生 C 库的外部函数接口绑定。为原始 C API 提供安全的 Rust 封装。

## 关键文件

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块重导出 |
| `opencv.rs` | OpenCV C API 绑定 — CvMat、模板匹配、颜色转换 |
| `tesseract.rs` | Tesseract C API 绑定 — OCR 引擎初始化和文字提取 |

## AI Agent 指南

### 在此目录工作
- 这些是原始 `extern "C"` 声明 — 签名必须与 DLL 导出完全匹配
- `OwnedCvMat` 和 `HeaderCvMat` 为 OpenCV 矩阵指针提供 RAII 封装
- OpenCV 使用旧版 C API（`cv*` 函数），非 C++ API
- Tesseract 绑定目标为 C API（`TessBaseAPI*` 函数）

### 测试要求
- 需要 DLL 在 PATH 中：`opencv_world4100.dll`、`tesseract53.dll`
- 通过上层 `matcher.rs` 和 `ocr.rs` 间接测试

### 常见模式
- 所有指针在封装为 `Option<Self>` 前检查 null
- RAII Drop 实现调用对应的 release/destroy 函数
- 常量与 OpenCV/Tesseract C 头文件值完全匹配

## 外部依赖

- OpenCV 4.10 C API（运行时 DLL）
- Tesseract 5.3 C API（运行时 DLL）

<!-- MANUAL: -->
