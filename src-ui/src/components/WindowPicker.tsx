import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WindowInfo {
  hwnd: number;
  title: string;
  process_name: string;
}

interface WindowPickerProps {
  onSelect: (hwnd: number, title: string) => void;
  selected: { hwnd: number; title: string } | null;
}

export function WindowPicker({ onSelect, selected }: WindowPickerProps) {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [filter, setFilter] = useState('');
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);

  async function refresh() {
    setLoading(true);
    try {
      const list: WindowInfo[] = await invoke('list_windows');
      setWindows(list);
    } catch {
      setWindows([]);
    }
    setLoading(false);
  }

  useEffect(() => { refresh(); }, []);

  const filtered = windows.filter((w) => {
    const q = filter.toLowerCase();
    return (
      w.title.toLowerCase().includes(q) ||
      w.process_name.toLowerCase().includes(q)
    );
  });

  return (
    <div className="relative">
      <label className="block text-sm text-gray-400 mb-1">Target Window</label>
      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => { setOpen(!open); if (!open) refresh(); }}
          className="flex-1 px-3 py-1.5 bg-gray-800 border border-gray-600 rounded text-sm text-left truncate"
        >
          {selected ? selected.title : 'Select a window...'}
        </button>
        <button
          type="button"
          onClick={refresh}
          disabled={loading}
          className="px-2 py-1.5 bg-gray-700 border border-gray-600 rounded text-sm hover:bg-gray-600 disabled:opacity-50"
          title="Refresh window list"
        >
          {loading ? '...' : '↻'}
        </button>
      </div>

      {open && (
        <div className="absolute z-50 mt-1 w-full bg-gray-800 border border-gray-600 rounded shadow-lg max-h-64 overflow-hidden flex flex-col">
          <input
            type="text"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Search..."
            className="px-3 py-1.5 bg-gray-900 border-b border-gray-600 text-sm outline-none"
            autoFocus
          />
          <ul className="overflow-y-auto flex-1">
            {filtered.length === 0 && (
              <li className="px-3 py-2 text-sm text-gray-500">No windows found</li>
            )}
            {filtered.map((w) => (
              <li
                key={w.hwnd}
                onClick={() => { onSelect(w.hwnd, w.title); setOpen(false); }}
                className={`px-3 py-2 text-sm cursor-pointer hover:bg-gray-700 ${
                  selected?.hwnd === w.hwnd ? 'bg-gray-700' : ''
                }`}
              >
                <div className="truncate font-medium">{w.title}</div>
                <div className="truncate text-xs text-gray-400">{w.process_name}</div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
