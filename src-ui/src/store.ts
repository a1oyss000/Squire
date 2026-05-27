import { create } from 'zustand';

export interface TaskInfo {
  name: string;
  enabled: boolean;
  description?: string;
}

export interface TaskResult {
  task_name: string;
  state: 'pending' | 'running' | 'success' | 'failed' | 'skipped' | 'cancelled';
  message?: string;
  duration_ms: number;
}

interface AppStore {
  tasks: TaskInfo[];
  results: TaskResult[];
  isRunning: boolean;
  setTasks: (tasks: TaskInfo[]) => void;
  setResults: (results: TaskResult[]) => void;
  setRunning: (running: boolean) => void;
  reorderTasks: (from: number, to: number) => void;
  toggleTask: (name: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  tasks: [],
  results: [],
  isRunning: false,
  setTasks: (tasks) => set({ tasks }),
  setResults: (results) => set({ results }),
  setRunning: (running) => set({ isRunning: running }),
  reorderTasks: (from, to) =>
    set((state) => {
      const tasks = [...state.tasks];
      const [moved] = tasks.splice(from, 1);
      tasks.splice(to, 0, moved);
      return { tasks };
    }),
  toggleTask: (name) =>
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.name === name ? { ...t, enabled: !t.enabled } : t
      ),
    })),
}));
