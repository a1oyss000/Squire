import { useAppStore } from '../store';

export function TaskList() {
  const { tasks, toggleTask } = useAppStore();

  if (tasks.length === 0) {
    return (
      <div className="text-gray-500 text-sm text-center py-8">
        No tasks found. Add YAML files to the tasks/ directory.
      </div>
    );
  }

  return (
    <ul className="space-y-2">
      {tasks.map((task) => (
        <li
          key={task.name}
          className="flex items-center gap-3 p-3 rounded bg-gray-800 hover:bg-gray-750 cursor-grab"
        >
          <input
            type="checkbox"
            checked={task.enabled}
            onChange={() => toggleTask(task.name)}
            className="w-4 h-4 rounded"
          />
          <div className="flex-1 min-w-0">
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
