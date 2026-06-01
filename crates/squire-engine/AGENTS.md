<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# squire-engine

## 概述
任务执行引擎。加载 YAML 任务定义，调度对目标窗口的执行，运行带有 retry/skip/pause 策略的步骤，并报告事件。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | 依赖：所有兄弟 crate + tokio + serde_yaml |
| `src/lib.rs` | 模块导出：config、runner、scheduler、state |
| `src/config.rs` | 任务/步骤/动作数据模型和 YAML 反序列化 |
| `src/runner.rs` | 步骤执行逻辑、目标解析、重试循环 |
| `src/scheduler.rs` | 引擎生命周期 — 通过 channel 生成异步任务运行器 |
| `src/state.rs` | `TaskResult` 和 `TaskState` 类型 |

## AI Agent 指南

### 在此目录工作
- `config.rs` 定义 YAML schema — 修改会影响任务文件格式
- `runner.rs` 是热路径：截图 → 匹配/OCR → 输入 → 报告
- `scheduler.rs` 通过 `mpsc`/`watch` channel 管理引擎生命周期
- 引擎通过 `EngineCommand`（Start/Cancel）和 `EngineEvent` 通信

### 测试要求
- `cargo test -p squire-engine`
- 配置解析测试可在任何平台运行；runner 测试需要 Windows + DLL

### 常见模式
- `RunContext` 打包输入后端、窗口句柄、取消信号和基础目录
- 步骤将目标解析为 `Point` 然后分发给 `InputBackend`
- 模板路径通过 `ctx.base_dir.join(path)` 在运行时解析
- 失败处理：每步的 `on_fail` 覆盖任务级 `fail_strategy`

## 依赖

### 内部
- `squire-error` — 错误类型
- `squire-vision` — 截图 + 匹配 + OCR
- `squire-input` — 输入模拟

### 外部
- `tokio` + `tokio-util` — 异步运行时和 channel
- `serde` + `serde_yaml` — YAML 配置反序列化
- `tracing` — 结构化日志

<!-- MANUAL: -->
