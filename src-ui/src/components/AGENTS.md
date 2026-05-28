<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# components

## Purpose
React UI components for the Squire desktop app. Each component handles a specific feature area.

## Key Files

| File | Description |
|------|-------------|
| `WindowPicker.tsx` | Dropdown to select a target window from the system window list |
| `TaskList.tsx` | Sortable list of automation tasks with enable/disable toggles |
| `TaskConfig.tsx` | YAML config viewer/editor for a selected task |
| `ExecutionStatus.tsx` | Real-time execution log display |

## For AI Agents

### Working In This Directory
- Components use Tauri `invoke()` for backend calls and `listen()` for events
- Styling is Tailwind utility classes only — no separate CSS files
- `WindowPicker` uses a custom dropdown with search filtering
- `TaskList` supports drag-and-drop reordering via @dnd-kit

### Common Patterns
- Props interfaces defined above the component export
- Local state via `useState`, global state via `useAppStore` from Zustand
- Async operations wrapped in try/catch with loading states

## Dependencies

### Internal
- `../store.ts` — Zustand store for shared state

### External
- `@tauri-apps/api/core` — `invoke` for IPC commands
- `@dnd-kit/core`, `@dnd-kit/sortable` — drag-and-drop

<!-- MANUAL: -->
