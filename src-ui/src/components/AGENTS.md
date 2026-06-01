<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# components

## 概述
Squire 桌面应用的 React UI 组件。每个组件负责一个特定功能区域。

## 关键文件

| 文件 | 说明 |
|------|------|
| `WindowPicker.tsx` | 从系统窗口列表选择目标窗口的下拉框 |
| `TaskList.tsx` | 可排序的自动化任务列表，带启用/禁用开关 |
| `TaskConfig.tsx` | 选中任务的 YAML 配置查看器/编辑器 |
| `ExecutionStatus.tsx` | 实时执行日志显示 |

## AI Agent 指南

### 在此目录工作
- 组件使用 Tauri `invoke()` 调用后端，使用 `listen()` 监听事件
- 样式仅使用 Tailwind 工具类 — 无独立 CSS 文件
- `WindowPicker` 使用带搜索过滤的自定义下拉框
- `TaskList` 通过 @dnd-kit 支持拖拽排序

### 常见模式
- Props 接口定义在组件导出上方
- 局部状态用 `useState`，全局状态用 Zustand 的 `useAppStore`
- 异步操作包裹在 try/catch 中并带有 loading 状态

## 依赖

### 内部
- `../store.ts` — Zustand store 共享状态

### 外部
- `@tauri-apps/api/core` — `invoke` 用于 IPC 命令
- `@dnd-kit/core`、`@dnd-kit/sortable` — 拖拽排序

<!-- MANUAL: -->
