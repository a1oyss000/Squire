import { useRef, useState } from 'react';
import { useAppStore } from '../store';

interface TaskListProps {
  selectedTask?: string | null;
  onSelectTask?: (name: string) => void;
}

export function TaskList({ selectedTask, onSelectTask }: TaskListProps) {
  const { tasks, toggleTask, reorderTasks } = useAppStore();
  const dragIndex = useRef<number | null>(null);
  const [dragOver, setDragOver] = useState<{ index: number; position: 'top' | 'bottom' } | null>(null);
  const [dragging, setDragging] = useState<number | null>(null);

  if (tasks.length === 0) {
    return (
      <div className="text-gray-500 text-sm text-center py-8">
        No tasks found. Add YAML files to the tasks/ directory.
      </div>
    );
  }

  function handleDragStart(e: React.DragEvent, index: number) {
    dragIndex.current = index;
    setDragging(index);
    e.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(e: React.DragEvent, index: number) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const midY = rect.top + rect.height / 2;
    setDragOver({ index, position: e.clientY < midY ? 'top' : 'bottom' });
  }

  function handleDragLeave() {
    setDragOver(null);
  }

  function handleDrop(e: React.DragEvent, toIndex: number) {
    e.preventDefault();
    const fromIndex = dragIndex.current;
    if (fromIndex === null || fromIndex === toIndex) {
      setDragOver(null);
      setDragging(null);
      return;
    }

    const position = dragOver?.position;
    let target = toIndex;
    if (position === 'bottom' && toIndex < tasks.length - 1) {
      target = toIndex + 1;
    }
    if (fromIndex !== target) {
      reorderTasks(fromIndex, target > fromIndex ? target - 1 : target);
    }

    const newOrder = (() => {
      const arr = [...useAppStore.getState().tasks];
      return arr.map((t) => t.name);
    })();

    (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_task_order', { order: newOrder });
      } catch {}
    })();

    setDragOver(null);
    setDragging(null);
    dragIndex.current = null;
  }

  function handleDragEnd() {
    setDragOver(null);
    setDragging(null);
    dragIndex.current = null;
  }

  return (
    <ul className="space-y-2">
      {tasks.map((task, index) => (
        <li
          key={task.name}
          draggable={true}
          onDragStart={(e) => handleDragStart(e, index)}
          onDragOver={(e) => handleDragOver(e, index)}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, index)}
          onDragEnd={handleDragEnd}
          className={[
            'flex items-center gap-3 p-3 rounded select-none',
            selectedTask === task.name ? 'bg-gray-600' : 'bg-gray-800 hover:bg-gray-750',
            dragging === index ? 'opacity-50' : '',
            dragOver?.index === index && dragOver.position === 'top'
              ? 'border-t-2 border-blue-400'
              : '',
            dragOver?.index === index && dragOver.position === 'bottom'
              ? 'border-b-2 border-blue-400'
              : '',
          ]
            .filter(Boolean)
            .join(' ')}
        >
          <span
            className="text-gray-500 cursor-grab active:cursor-grabbing text-lg leading-none select-none"
            title="Drag to reorder"
          >
            ⠿
          </span>
          <input
            type="checkbox"
            checked={task.enabled}
            onChange={() => toggleTask(task.name)}
            className="w-4 h-4 rounded"
          />
          <div
            className="flex-1 min-w-0 cursor-pointer"
            onClick={() => onSelectTask?.(task.name)}
          >
            <div className="text-sm font-medium truncate">{task.name}</div>
            {task.description && (
              <div className="text-xs text-gray-400 truncate">
                {task.description}
              </div>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}
