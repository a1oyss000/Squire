<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# squire-input

## 概述
输入模拟后端。提供基于 trait 的平台特定输入方法抽象（Win32 API、ADB 用于 Android 模拟器）。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | 依赖：windows crate（键盘/鼠标 API） |
| `src/lib.rs` | `InputBackend` trait 定义和 `Point` 结构体 |
| `src/winapi.rs` | Win32 `SendInput` 实现 |
| `src/adb.rs` | 基于 ADB 的 Android 模拟器输入 |

## AI Agent 指南

### 在此目录工作
- 为新输入方式实现 `InputBackend` trait
- Trait 方法：`click`、`double_click`、`drag`、`key_press`
- `Point { x: i32, y: i32 }` 使用屏幕坐标

### 测试要求
- 输入测试本质上有副作用 — 在专用窗口上测试
- `cargo build -p squire-input` 验证编译

### 常见模式
- 后端实现为 `Send + Sync` 以便跨异步任务使用
- Win32 后端使用 `SendInput` 配合 `INPUT_MOUSE` / `INPUT_KEYBOARD`

## 依赖

### 内部
- `squire-error` — 错误类型

### 外部
- `windows` 0.58 — `Win32_UI_Input_KeyboardAndMouse`、`Win32_UI_WindowsAndMessaging`

<!-- MANUAL: -->
