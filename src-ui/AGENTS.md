<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-06-02 -->

# src-ui

## 概述
Squire 桌面应用的 React 前端。提供任务管理、窗口选择、执行控制和状态监控，通过 Tauri IPC 与后端通信。

## 关键文件

| 文件 | 说明 |
|------|------|
| `package.json` | 依赖和脚本（dev、build、lint） |
| `vite.config.ts` | Vite 构建配置 |
| `tsconfig.json` | TypeScript 配置 |
| `index.html` | HTML 入口 |

## 子目录

| 目录 | 用途 |
|------|------|
| `src/` | 应用源码（见 `src/AGENTS.md`） |
| `public/` | 原样提供的静态资源 |

## AI Agent 指南

### 在此目录工作
- `npm run dev` 启动 Vite 开发服务器（配合 `cargo tauri dev` 使用）
- 样式使用 Tailwind CSS 4（工具类，无需配置文件）
- 状态管理使用 Zustand（单 store 在 `src/store.ts`）

### 测试要求
- `npm run lint` 进行 ESLint 检查
- `npm run build` 验证 TypeScript 编译 + 生产构建

## 外部依赖

- React 19 — UI 框架
- Zustand 5 — 状态管理
- @tauri-apps/api 2 — 与 Rust 后端的 IPC 桥接
- @dnd-kit — 任务拖拽排序
- Tailwind CSS 4 — 工具优先的样式方案
- Vite 8 — 构建工具

<!-- MANUAL: -->
