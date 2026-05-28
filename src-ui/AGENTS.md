<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# src-ui

## Purpose
React frontend for the Squire desktop app. Provides task management, window selection, execution control, and status monitoring via Tauri IPC.

## Key Files

| File | Description |
|------|-------------|
| `package.json` | Dependencies and scripts (dev, build, lint) |
| `vite.config.ts` | Vite build configuration |
| `tsconfig.json` | TypeScript configuration |
| `index.html` | HTML entry point |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | Application source (see `src/AGENTS.md`) |
| `public/` | Static assets served as-is |

## For AI Agents

### Working In This Directory
- `npm run dev` starts the Vite dev server (use with `cargo tauri dev`)
- Styling uses Tailwind CSS 4 (utility classes, no config file needed)
- State management via Zustand (single store in `src/store.ts`)

### Testing Requirements
- `npm run lint` for ESLint checks
- `npm run build` to verify TypeScript compilation + production bundle

## Dependencies

### External
- React 19 — UI framework
- Zustand 5 — state management
- @tauri-apps/api 2 — IPC bridge to Rust backend
- @dnd-kit — drag-and-drop for task reordering
- Tailwind CSS 4 — utility-first styling
- Vite 8 — build tool

<!-- MANUAL: -->
