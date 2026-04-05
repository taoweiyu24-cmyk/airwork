import { useState, useEffect, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Bell, Search, User } from 'lucide-react';
import { search } from '../../services/api';
import type { SearchResult } from '../../types/domain';

interface HeaderProps {
  title: string;
  subtitle?: string;
}

export default function Header({ title, subtitle }: HeaderProps) {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!query.trim()) { setResults([]); setOpen(false); return; }
    setLoading(true);
    setOpen(true);
    const timer = setTimeout(async () => {
      try { setResults(await search(query.trim(), 10)); }
      catch { setResults([]); }
      finally { setLoading(false); }
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const handleResultClick = useCallback((_id: string) => {
    setOpen(false); setQuery(''); navigate('/utility/items');
  }, [navigate]);

  const scoreBadge = (s: number) =>
    s >= 0.8 ? 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300'
    : s >= 0.5 ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-300'
    : 'bg-gray-100 text-gray-600 dark:bg-gray-600 dark:text-gray-300';

  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-gray-200 bg-white px-6 dark:border-gray-700 dark:bg-gray-800">
      {/* Left: title */}
      <div className="flex flex-col justify-center">
        <h1 className="text-sm font-semibold text-gray-900 dark:text-white leading-tight">
          {title}
        </h1>
        {subtitle && (
          <p className="text-xs text-gray-500 dark:text-gray-400 leading-tight">{subtitle}</p>
        )}
      </div>

      {/* Center: search */}
      <div className="mx-6 hidden flex-1 max-w-sm sm:flex" ref={containerRef}>
        <div className="relative w-full">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Escape') setOpen(false); }}
            onFocus={() => { if (query.trim() && results.length > 0) setOpen(true); }}
            placeholder="搜索..."
            className="w-full rounded-lg border border-gray-200 bg-gray-50 py-1.5 pl-9 pr-3 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-blue-500/20 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-blue-400 dark:focus:bg-gray-700"
          />

          {open && (
            <div className="absolute left-0 right-0 top-full z-50 mt-1 max-h-72 overflow-y-auto rounded-lg border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-800">
              {loading && (
                <p className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">搜索中...</p>
              )}
              {!loading && results.length === 0 && (
                <p className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">无结果</p>
              )}
              {!loading &&
                results.map((r) => (
                  <button
                    key={r.id}
                    type="button"
                    className="flex w-full items-start gap-2 px-4 py-2.5 text-left hover:bg-gray-50 dark:hover:bg-gray-700/60"
                    onClick={() => handleResultClick(r.id)}
                  >
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-gray-900 dark:text-white">
                        {r.title}
                      </p>
                      {r.snippet && (
                        <p className="mt-0.5 truncate text-xs text-gray-500 dark:text-gray-400">
                          {r.snippet.length > 80 ? `${r.snippet.slice(0, 80)}...` : r.snippet}
                        </p>
                      )}
                    </div>
                    <span
                      className={`mt-0.5 shrink-0 rounded px-1.5 py-0.5 text-xs font-medium ${scoreBadge(r.score)}`}
                    >
                      {(r.score * 100).toFixed(0)}%
                    </span>
                  </button>
                ))}
            </div>
          )}
        </div>
      </div>

      {/* Right: actions */}
      <div className="flex items-center gap-2">
        <button
          className="relative rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200"
          aria-label="通知"
        >
          <Bell className="h-4 w-4" />
          <span className="absolute right-1.5 top-1.5 h-1.5 w-1.5 rounded-full bg-blue-600" />
        </button>

        <div
          className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400"
          aria-label="用户"
        >
          <User className="h-4 w-4" />
        </div>
      </div>
    </header>
  );
}
