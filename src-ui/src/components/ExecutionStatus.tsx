import { useAppStore } from '../store';
import { useEffect, useRef } from 'react';

interface Props {
  logs?: string[];
}

export function ExecutionStatus({ logs = [] }: Props) {
  const { isRunning } = useAppStore();
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  function formatLog(msg: string): { text: string; color: string } {
    if (msg.startsWith('started:')) {
      return { text: `Task started: ${msg.slice(8)}`, color: 'text-blue-400' };
    }
    if (msg.startsWith('step_ok:')) {
      const parts = msg.slice(8).split(':');
      return { text: `  Step ${parseInt(parts[1]) + 1} OK (${parts[0]})`, color: 'text-green-400' };
    }
    if (msg.startsWith('step_fail:')) {
      const parts = msg.slice(10).split(':');
      return { text: `  Step ${parseInt(parts[1]) + 1} FAILED: ${parts.slice(2).join(':')}`, color: 'text-red-400' };
    }
    if (msg.startsWith('completed:')) {
      const parts = msg.slice(10).split(':');
      return { text: `Task done: ${parts[0]} [${parts[1]}]`, color: 'text-gray-300' };
    }
    if (msg === 'done') {
      return { text: 'All tasks finished.', color: 'text-yellow-300' };
    }
    if (msg.startsWith('Error:')) {
      return { text: msg, color: 'text-red-400' };
    }
    return { text: msg, color: 'text-gray-400' };
  }

  return (
    <div>
      <h2 className="text-lg font-semibold mb-4">Execution Log</h2>

      {isRunning && (
        <div className="flex items-center gap-2 mb-4 text-yellow-400">
          <div className="w-2 h-2 rounded-full bg-yellow-400 animate-pulse" />
          <span className="text-sm">Running...</span>
        </div>
      )}

      {logs.length === 0 && !isRunning && (
        <div className="text-gray-500 text-sm">
          Select a window and click "Start" to begin.
        </div>
      )}

      {logs.length > 0 && (
        <div className="bg-gray-800 rounded p-3 font-mono text-xs space-y-1 max-h-[600px] overflow-y-auto">
          {logs.map((msg, i) => {
            const { text, color } = formatLog(msg);
            return (
              <div key={i} className={color}>{text}</div>
            );
          })}
          <div ref={logEndRef} />
        </div>
      )}
    </div>
  );
}
