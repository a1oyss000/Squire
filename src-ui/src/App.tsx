import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from './store';
import { TaskList } from './components/TaskList';
import { ExecutionStatus } from './components/ExecutionStatus';
import { TaskConfig } from './components/TaskConfig';
import { WindowPicker } from './components/WindowPicker';
import './index.css';

function App() {
  const { tasks, isRunning, setTasks, setRunning } = useAppStore();
  const [selectedWindow, setSelectedWindow] = useState<{ hwnd: number; title: string } | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [selectedTask, setSelectedTask] = useState<string | null>(null);

  useEffect(() => {
    loadTasks();
    const unlistenPromise = listen<string>('engine-event', (event) => {
      const msg = event.payload;
      setLogs((prev) => [...prev, msg]);
      if (msg === 'done') {
        setRunning(false);
      }
    });
    return () => { unlistenPromise.then((fn) => fn()); };
  }, []);

  async function loadTasks() {
    try {
      const names: string[] = await invoke('get_tasks');
      setTasks(names.map((name) => ({ name, enabled: true })));
    } catch {
      setTasks([
        { name: 'nikke-daily-mail', enabled: true, description: 'NIKKE Daily Mail' },
        { name: 'nikke-shop', enabled: false, description: 'NIKKE Shop' },
      ]);
    }
  }

  async function handleStart() {
    if (!selectedWindow) return;
    setRunning(true);
    setLogs([]);
    try {
      const enabledTasks = tasks.filter((t) => t.enabled).map((t) => t.name);
      await invoke('start_execution', {
        hwnd: selectedWindow.hwnd,
        taskNames: enabledTasks,
      });
    } catch (e: any) {
      setLogs((prev) => [...prev, `Error: ${e}`]);
      setRunning(false);
    }
  }

  async function handleStop() {
    try {
      await invoke('stop_execution');
    } catch {}
    setRunning(false);
  }

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 flex flex-col">
      <header className="border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-bold">Squire</h1>
          <span className="text-sm text-gray-400">Game Automation</span>
        </div>
      </header>

      <main className="flex-1 flex">
        <div className="w-80 border-r border-gray-700 p-4 overflow-y-auto">
          <div className="mb-4">
            <WindowPicker
              selected={selectedWindow}
              onSelect={(hwnd, title) => setSelectedWindow({ hwnd, title })}
            />
          </div>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Tasks</h2>
            <button
              onClick={isRunning ? handleStop : handleStart}
              disabled={!isRunning && !selectedWindow}
              className={`px-4 py-1.5 rounded text-sm font-medium disabled:opacity-50 ${
                isRunning
                  ? 'bg-red-600 hover:bg-red-700'
                  : 'bg-green-600 hover:bg-green-700'
              }`}
            >
              {isRunning ? 'Stop' : 'Start'}
            </button>
          </div>
          <TaskList selectedTask={selectedTask} onSelectTask={setSelectedTask} />
        </div>

        <div className="flex-1 p-4 overflow-y-auto">
          {selectedTask ? (
            <TaskConfig taskName={selectedTask} onClose={() => setSelectedTask(null)} />
          ) : (
            <ExecutionStatus logs={logs} />
          )}
        </div>
      </main>
    </div>
  );
}

export default App;

