<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# templates

## 概述
视觉匹配器用于定位屏幕 UI 元素的图片模板文件。按游戏/应用分类组织。

## 子目录

| 目录 | 用途 |
|------|------|
| `nikke/` | NIKKE 游戏自动化的模板图片 |

## AI Agent 指南

### 在此目录工作
- 模板为从游戏截图裁剪的 PNG 图片
- 由任务 YAML 文件通过相对路径引用（如 `templates/nikke/mail_button.png`）
- 保持模板小而有辨识度以确保匹配可靠
- 阈值默认 0.8 — 对变化较大的 UI 元素可降低

<!-- MANUAL: -->
