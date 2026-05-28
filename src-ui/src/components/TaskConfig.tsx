import { useEffect, useState } from 'react';

interface TaskConfigProps {
  taskName: string;
  onClose: () => void;
}

interface ParsedParams {
  name?: string;
  retry_count?: number | string;
  fail_strategy?: string;
}

function parseParams(yaml: string): ParsedParams {
  const params: ParsedParams = {};
  const nameMatch = yaml.match(/^name:\s*(.+)$/m);
  const retryMatch = yaml.match(/^retry_count:\s*(.+)$/m);
  const failMatch = yaml.match(/^fail_strategy:\s*(.+)$/m);
  if (nameMatch) params.name = nameMatch[1].trim();
  if (retryMatch) params.retry_count = retryMatch[1].trim();
  if (failMatch) params.fail_strategy = failMatch[1].trim();
  return params;
}

export function TaskConfig({ taskName, onClose }: TaskConfigProps) {
  const [yaml, setYaml] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setYaml(null);
    (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const content: string = await invoke('get_task_config', { name: taskName });
        setYaml(content);
      } catch (e: any) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, [taskName]);

  const params = yaml ? parseParams(yaml) : {};

  return (
    <div className="bg-gray-800 border border-gray-700 rounded p-4 h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold">{taskName}</h2>
        <button
          onClick={onClose}
          className="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded"
        >
          Close
        </button>
      </div>

      {loading && (
        <div className="text-gray-400 text-sm">Loading config...</div>
      )}

      {error && (
        <div className="text-red-400 text-sm">Error: {error}</div>
      )}

      {yaml && !loading && (
        <>
          {(params.name || params.retry_count !== undefined || params.fail_strategy) && (
            <div className="mb-4 grid grid-cols-3 gap-3">
              {params.name && (
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-xs text-gray-400 mb-1">name</div>
                  <div className="text-sm font-mono">{params.name}</div>
                </div>
              )}
              {params.retry_count !== undefined && (
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-xs text-gray-400 mb-1">retry_count</div>
                  <div className="text-sm font-mono">{params.retry_count}</div>
                </div>
              )}
              {params.fail_strategy && (
                <div className="bg-gray-700 rounded p-2">
                  <div className="text-xs text-gray-400 mb-1">fail_strategy</div>
                  <div className="text-sm font-mono">{params.fail_strategy}</div>
                </div>
              )}
            </div>
          )}
          <pre className="flex-1 overflow-auto bg-gray-900 rounded p-3 text-sm font-mono text-gray-200 whitespace-pre-wrap">
            {yaml}
          </pre>
        </>
      )}
    </div>
  );
}
