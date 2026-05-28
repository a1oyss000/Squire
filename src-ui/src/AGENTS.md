<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# src (UI)

## Purpose
React application source. Contains the main app component, state store, and feature-specific components for the Squire desktop UI.

## Key Files

| File | Description |
|------|-------------|
| `main.tsx` | React entry point — renders App into DOM |
| `App.tsx` | Root component: layout, window picker, task list, execution panel |
| `store.ts` | Zustand store: tasks, running state, shared app state |
| `index.css` | Tailwind CSS imports |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `components/` | Reusable UI components (see `components/AGENTS.md`) |
| `assets/` | Static assets (images, SVGs) |

## For AI Agents

### Working In This Directory
- Components communicate via Zustand store (`useAppStore`) and props
- Tauri IPC calls use `invoke('command_name', { args })` from `@tauri-apps/api/core`
- Event listening via `listen<T>('event-name', callback)` from `@tauri-apps/api/event`

### Common Patterns
- Functional components with hooks
- Tailwind utility classes for all styling (no CSS modules)
- State flows: Zustand for global, `useState` for local component state

<!-- MANUAL: -->
