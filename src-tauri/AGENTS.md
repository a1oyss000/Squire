<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# src-tauri

## 概述
Tauri 2 应用外壳。定义暴露给前端的 IPC 命令，管理应用状态，并将各引擎 crate 组装在一起。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | 应用依赖 — 所有 workspace crate + Tauri |
| `src/main.rs` | 入口：Tauri 初始化、命令处理器、日志配置 |
| `tauri.conf.json` | Tauri 配置（窗口大小、应用名、权限） |
| `build.rs` | Tauri 构建脚本 |

## 子目录

| 目录 | 用途 |
|------|------|
| `src/` | Tauri 应用 Rust 源码（见 `src/AGENTS.md`） |
| `capabilities/` | Tauri 权限能力文件 |
| `icons/` | 应用图标 |
| `gen/` | 自动生成的 Tauri schema（勿编辑） |

## AI Agent 指南

### 在此目录工作
- 在 `src/main.rs` 中添加新 IPC 命令并在 `generate_handler![]` 中注册
- 使用 `State<AppState>` 在命令中访问共享状态
- 日志通过 `tracing-appender` 写入每日滚动文件

### 测试要求
- `cargo build -p squire-app` 验证编译
- IPC 命令通过前端或集成测试验证

<!-- MANUAL: -->
