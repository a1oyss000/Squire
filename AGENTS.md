<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# Squire

## 概述
Windows 游戏自动化工具，基于 Tauri 构建。通过计算机视觉（模板匹配、OCR）和模拟输入（点击、拖拽、按键）对目标窗口执行 YAML 定义的任务序列。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | Workspace manifest，定义所有 crate 和共享依赖 |
| `tauri.conf.json` | Tauri 应用配置（位于 `src-tauri/`） |
| `README.md` | 项目说明文档 |

## 子目录

| 目录 | 用途 |
|------|------|
| `crates/` | Rust 库 crate（见 `crates/AGENTS.md`） |
| `src-tauri/` | Tauri 应用外壳和 IPC 命令（见 `src-tauri/AGENTS.md`） |
| `src-ui/` | React 前端任务管理界面（见 `src-ui/AGENTS.md`） |
| `tasks/` | 运行时加载的 YAML 任务定义（见 `tasks/AGENTS.md`） |
| `templates/` | 视觉匹配用的图片模板（见 `templates/AGENTS.md`） |
| `scripts/` | 辅助工具脚本（见 `scripts/AGENTS.md`） |
| `docs/` | 项目文档（见 `docs/AGENTS.md`） |

## AI Agent 指南

### 在此目录工作
- 这是一个 Cargo workspace — 在根目录运行 `cargo build` 构建所有 crate
- Tauri 应用（`src-tauri`）依赖所有 workspace crate
- Windows 专属功能通过 `cfg(windows)` 门控 — 非 Windows 构建可编译但会 stub 掉视觉/输入模块

### 测试要求
- `cargo test` 运行所有 crate 测试
- `cargo clippy` 进行 lint 检查
- 视觉/输入测试需要 Windows 环境和显示器访问

### 常见模式
- 错误类型集中在 `squire-error`，通过 `squire_error::Result<T>` 重导出
- 异步运行时为 Tokio；引擎使用 channel（`mpsc`、`watch`）控制流程
- 任务定义为 YAML 文件，通过 serde 反序列化
- YAML 中的模板路径在运行时相对于项目根目录解析

## 外部依赖

- Tauri 2.x — 桌面应用框架
- Tokio — 异步运行时
- OpenCV 4.10 (DLL) — 通过 FFI 进行模板匹配
- Tesseract 5.3 (DLL) — 通过 FFI 进行 OCR
- Windows crate 0.58 — Win32 API 绑定

<!-- MANUAL: -->
