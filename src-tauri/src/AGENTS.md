<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# src (Tauri)

## 概述
Tauri 应用的 Rust 源码。包含入口、IPC 命令处理器和应用状态管理。

## 关键文件

| 文件 | 说明 |
|------|------|
| `main.rs` | 应用入口：Tauri 初始化、日志配置、命令注册、引擎生命周期 |

## AI Agent 指南

### 在此目录工作
- 所有 IPC 命令为 `#[tauri::command]` 函数，在 `generate_handler![]` 中注册
- `AppState` 持有：任务目录路径、执行结果、引擎命令 sender
- 日志：通过 `tracing-appender` 写入每日滚动文件，文件用 JSON 格式，控制台用紧凑格式
- 任务目录在开发时解析为项目根的 `tasks/`，生产环境为资源目录

### 常见模式
- 命令返回 `Result<T, String>` 用于 Tauri IPC 序列化
- `safe_task_path()` 验证任务名以防止路径遍历
- 引擎启动：加载 YAML → 创建引擎 → 生成事件转发任务
- 引擎停止：通过存储的 channel sender 发送 `EngineCommand::Cancel`

<!-- MANUAL: -->
