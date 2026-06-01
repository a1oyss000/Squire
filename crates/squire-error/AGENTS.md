<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# squire-error

## 概述
整个 workspace 的共享错误类型。定义 `SquireError` 枚举和所有 crate 使用的 `Result<T>` 类型别名。

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | 唯一依赖：`thiserror` |
| `src/lib.rs` | `SquireError` 枚举，包含各子系统的变体 |

## AI Agent 指南

### 在此目录工作
- 引入新的失败模式时在此添加新的错误变体
- 所有变体通过 thiserror 的 `#[error("...")]` 实现 Display
- 其他 crate 依赖此 crate — 修改会影响整个 workspace

### 常见模式
- 变体命名：`SubsystemName(String)` 用于通用错误
- 结构化变体（如 `MatchFailed { confidence, threshold }`）用于可操作的错误
- `#[from]` 用于从标准类型（如 `std::io::Error`）自动转换

<!-- MANUAL: -->
