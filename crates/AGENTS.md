<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# crates

## 概述
核心自动化逻辑的 Rust 库 crate 容器。每个 crate 职责单一，由 Tauri 应用消费。

## 子目录

| 目录 | 用途 |
|------|------|
| `squire-error/` | 共享错误枚举和 Result 类型（见 `squire-error/AGENTS.md`） |
| `squire-vision/` | 屏幕截图、模板匹配、OCR（见 `squire-vision/AGENTS.md`） |
| `squire-input/` | 输入模拟后端（见 `squire-input/AGENTS.md`） |
| `squire-engine/` | 任务执行引擎 — 配置、运行器、调度器（见 `squire-engine/AGENTS.md`） |

## AI Agent 指南

### 在此目录工作
- 每个 crate 独立；新增 crate 后需在根 `Cargo.toml` 的 workspace members 中注册
- 依赖关系：`squire-error` ← `squire-vision`、`squire-input` ← `squire-engine`

<!-- MANUAL: -->
