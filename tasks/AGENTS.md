<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# tasks

## 概述
引擎运行时加载的 YAML 任务定义文件。每个文件定义一个命名的自动化序列，包含步骤、目标和失败策略。

## 关键文件

| 文件 | 说明 |
|------|------|
| `nikke-daily-mail.yaml` | NIKKE 每日邮件收取任务 |

## 子目录

| 目录 | 用途 |
|------|------|
| `templates/` | 任务步骤引用的模板图片（见 `templates/AGENTS.md`） |

## AI Agent 指南

### 在此目录工作
- 任务文件使用 `crates/squire-engine/src/config.rs` 中定义的 schema
- 步骤中的模板路径在运行时相对于项目根目录解析
- 每个任务至少需要一个超时非零的步骤

### 常见模式
- 步骤目标类型：`template`（图片匹配）、`ocr`（文字查找）、`coordinate`（固定坐标）
- 动作：`click`、`wait`、`swipe`
- 失败策略：`retry`（默认）、`skip`、`pause`

<!-- MANUAL: -->
