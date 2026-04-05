import { useState, useEffect, useCallback } from 'react';
import {
  Mail,
  RefreshCw,
  Loader2,
  AlertCircle,
  ChevronDown,
  ChevronRight,
  Inbox,
} from 'lucide-react';
import Card from './ui/Card';
import Button from './ui/Button';
import Badge from './ui/Badge';
import type { MailAccount, MailMessage } from '../types/domain';
import * as api from '../services/api';

function formatDateTime(epoch: number): string {
  return new Date(epoch * 1000).toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export default function InboxView() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState<string>('');
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.listMailAccounts();
      setAccounts(data);
      if (data.length > 0 && !selectedAccountId) {
        setSelectedAccountId(data[0].id);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '加载邮箱账户失败';
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [selectedAccountId]);

  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  const handleSync = useCallback(async () => {
    if (!selectedAccountId) return;
    setSyncing(true);
    setError(null);
    try {
      const data = await api.syncMailInbox(selectedAccountId, 50);
      setMessages(data);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '同步邮件失败';
      setError(msg);
    } finally {
      setSyncing(false);
    }
  }, [selectedAccountId]);

  const handleToggleExpand = useCallback((id: string) => {
    setExpandedId((prev) => (prev === id ? null : id));
  }, []);

  if (loading) {
    return (
      <Card title="邮件收件箱">
        <div className="flex items-center justify-center gap-2 py-16">
          <Loader2 className="h-5 w-5 animate-spin text-gray-400" />
          <span className="text-sm text-gray-500 dark:text-gray-400">加载中...</span>
        </div>
      </Card>
    );
  }

  if (accounts.length === 0) {
    return (
      <Card title="邮件收件箱">
        <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
          <Mail className="h-12 w-12 text-gray-300 dark:text-gray-600" />
          <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
            请先在设置中配置邮箱账户
          </p>
        </div>
      </Card>
    );
  }

  return (
    <Card title="邮件收件箱">
      {/* Toolbar */}
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <select
          className="rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-700 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200"
          value={selectedAccountId}
          onChange={(e) => {
            setSelectedAccountId(e.target.value);
            setMessages([]);
            setExpandedId(null);
          }}
        >
          {accounts.map((a) => (
            <option key={a.id} value={a.id}>
              {a.name} ({a.email})
            </option>
          ))}
        </select>

        <Button variant="primary" size="sm" onClick={handleSync} disabled={syncing}>
          {syncing ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
          同步
        </Button>

        {messages.length > 0 && (
          <span className="ml-auto text-xs text-gray-500 dark:text-gray-400">
            共 {messages.length} 封
          </span>
        )}
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 flex items-center gap-2 rounded-lg bg-red-50 px-4 py-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
          <AlertCircle className="h-4 w-4 shrink-0" />
          {error}
        </div>
      )}

      {/* Syncing spinner */}
      {syncing && (
        <div className="flex items-center justify-center gap-2 py-12">
          <Loader2 className="h-5 w-5 animate-spin text-gray-400" />
          <span className="text-sm text-gray-500 dark:text-gray-400">正在同步...</span>
        </div>
      )}

      {/* Empty state after sync */}
      {!syncing && messages.length === 0 && (
        <div className="flex flex-col items-center justify-center gap-2 py-12 text-center">
          <Inbox className="h-12 w-12 text-gray-300 dark:text-gray-600" />
          <p className="text-sm text-gray-500 dark:text-gray-400">
            点击「同步」获取最新邮件
          </p>
        </div>
      )}

      {/* Message list */}
      {!syncing && messages.length > 0 && (
        <div className="divide-y divide-gray-100 dark:divide-gray-700">
          {messages.map((msg) => {
            const isExpanded = expandedId === msg.id;
            return (
              <div key={msg.id}>
                <button
                  className="flex w-full items-start gap-3 px-1 py-3 text-left hover:bg-gray-50 dark:hover:bg-gray-800/50"
                  onClick={() => handleToggleExpand(msg.id)}
                >
                  {isExpanded ? (
                    <ChevronDown className="mt-0.5 h-4 w-4 shrink-0 text-gray-400" />
                  ) : (
                    <ChevronRight className="mt-0.5 h-4 w-4 shrink-0 text-gray-400" />
                  )}

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span
                        className={[
                          'truncate text-sm',
                          msg.isRead
                            ? 'text-gray-600 dark:text-gray-400'
                            : 'font-semibold text-gray-900 dark:text-white',
                        ].join(' ')}
                      >
                        {msg.subject || '（无主题）'}
                      </span>
                      {!msg.isRead && <Badge variant="info">未读</Badge>}
                      {msg.workItemId && <Badge variant="success">已关联</Badge>}
                    </div>
                    <div className="mt-0.5 flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                      <span className="truncate">{msg.fromAddress}</span>
                      <span className="shrink-0">{formatDateTime(msg.receivedAt)}</span>
                    </div>
                  </div>
                </button>

                {isExpanded && (
                  <div className="mb-2 ml-7 rounded-lg bg-gray-50 p-4 text-sm text-gray-700 dark:bg-gray-800 dark:text-gray-300">
                    <div className="mb-2 text-xs text-gray-500 dark:text-gray-400">
                      <span className="font-medium">收件人：</span>
                      {msg.toAddresses.join(', ')}
                    </div>
                    <div className="whitespace-pre-wrap break-words">
                      {msg.bodyText || '（无正文内容）'}
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </Card>
  );
}
