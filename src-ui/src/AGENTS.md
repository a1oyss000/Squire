<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# src (UI)

## 概述
React 应用源码。包含主应用组件、状态 store 和 Squire 桌面 UI 的功能组件。

## 关键文件

| 文件 | 说明 |
|------|------|
| `main.tsx` | React 入口 — 将 App 渲染到 DOM |
| `App.tsx` | 根组件：布局、窗口选择器、任务列表、执行面板 |
| `store.ts` | Zustand store：任务、运行状态、共享应用状态 |
| `index.css` | Tailwind CSS 导入 |

## 子目录

| 目录 | 用途 |
|------|------|
| `components/` | 可复用 UI 组件（见 `components/AGENTS.md`） |
| `assets/` | 静态资源（图片、SVG） |

## AI Agent 指南

### 在此目录工作
- 组件通过 Zustand store（`useAppStore`）和 props 通信
- Tauri IPC 调用使用 `@tauri-apps/api/core` 的 `invoke('command_name', { args })`
- 事件监听使用 `@tauri-apps/api/event` 的 `listen<T>('event-name', callback)`

### 常见模式
- 函数式组件 + hooks
- 所有样式使用 Tailwind 工具类（无 CSS modules）
- 状态流：Zustand 管理全局状态，`useState` 管理组件局部状态

<!-- MANUAL: -->
