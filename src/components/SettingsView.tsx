import { useState, useEffect } from 'react';
import { Bot, Mail, Loader2, Sparkles, ServerCrash } from 'lucide-react';
import Card from './ui/Card';
import Badge from './ui/Badge';
import { listAiProfiles, listMailAccounts } from '../services/api';
import type { AiProfile, MailAccount } from '../types/domain';

type Tab = 'ai' | 'mail';

function maskApiKey(key: string): string {
  if (key.length <= 4) return '****';
  return '****' + key.slice(-4);
}

function EmptyState({ icon: Icon, message }: { icon: React.ComponentType<{ className?: string }>; message: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
      <Icon className="h-12 w-12 text-gray-300 dark:text-gray-600" />
      <p className="text-sm font-medium text-gray-500 dark:text-gray-400">{message}</p>
    </div>
  );
}

function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <ServerCrash className="h-12 w-12 text-red-300 dark:text-red-500" />
      <p className="text-sm font-medium text-red-600 dark:text-red-400">{message}</p>
      <button
        onClick={onRetry}
        className="text-sm text-blue-600 underline hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
      >
        重试
      </button>
    </div>
  );
}

function LoadingState() {
  return (
    <div className="flex items-center justify-center py-16">
      <Loader2 className="h-6 w-6 animate-spin text-gray-400 dark:text-gray-500" />
      <span className="ml-2 text-sm text-gray-500 dark:text-gray-400">加载中...</span>
    </div>
  );
}

function AiProfilesSection() {
  const [profiles, setProfiles] = useState<AiProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchProfiles = () => {
    setLoading(true);
    setError(null);
    listAiProfiles()
      .then(setProfiles)
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : '加载 AI 配置失败';
        setError(msg);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchProfiles(); }, []);

  if (loading) return <LoadingState />;
  if (error) return <ErrorState message={error} onRetry={fetchProfiles} />;
  if (profiles.length === 0) return <EmptyState icon={Sparkles} message="暂无 AI 配置" />;

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-gray-200 dark:border-gray-700">
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">名称</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">提供商</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">模型</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">API 密钥</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">Base URL</th>
            <th className="pb-3 font-medium text-gray-500 dark:text-gray-400">状态</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100 dark:divide-gray-700/50">
          {profiles.map((p) => (
            <tr key={p.id} className="group hover:bg-gray-50 dark:hover:bg-gray-700/30">
              <td className="py-3 pr-4 font-medium text-gray-900 dark:text-white">{p.name}</td>
              <td className="py-3 pr-4 text-gray-600 dark:text-gray-300">{p.provider}</td>
              <td className="py-3 pr-4">
                <code className="rounded bg-gray-100 px-1.5 py-0.5 text-xs text-gray-700 dark:bg-gray-700 dark:text-gray-300">
                  {p.model}
                </code>
              </td>
              <td className="py-3 pr-4 font-mono text-xs text-gray-500 dark:text-gray-400">
                {maskApiKey(p.apiKey)}
              </td>
              <td className="py-3 pr-4 max-w-[200px] truncate text-gray-500 dark:text-gray-400" title={p.baseUrl}>
                {p.baseUrl}
              </td>
              <td className="py-3">
                {p.isDefault ? (
                  <Badge variant="success">默认</Badge>
                ) : (
                  <Badge variant="default">可用</Badge>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function MailAccountsSection() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAccounts = () => {
    setLoading(true);
    setError(null);
    listMailAccounts()
      .then(setAccounts)
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : '加载邮箱账户失败';
        setError(msg);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => { fetchAccounts(); }, []);

  if (loading) return <LoadingState />;
  if (error) return <ErrorState message={error} onRetry={fetchAccounts} />;
  if (accounts.length === 0) return <EmptyState icon={Mail} message="暂无邮箱账户" />;

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-gray-200 dark:border-gray-700">
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">名称</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">邮箱</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">提供商</th>
            <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">IMAP 主机</th>
            <th className="pb-3 font-medium text-gray-500 dark:text-gray-400">状态</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100 dark:divide-gray-700/50">
          {accounts.map((a) => (
            <tr key={a.id} className="group hover:bg-gray-50 dark:hover:bg-gray-700/30">
              <td className="py-3 pr-4 font-medium text-gray-900 dark:text-white">{a.name}</td>
              <td className="py-3 pr-4 text-gray-600 dark:text-gray-300">{a.email}</td>
              <td className="py-3 pr-4 text-gray-600 dark:text-gray-300">{a.provider}</td>
              <td className="py-3 pr-4">
                <code className="rounded bg-gray-100 px-1.5 py-0.5 text-xs text-gray-700 dark:bg-gray-700 dark:text-gray-300">
                  {a.imapHost}:{a.imapPort}
                </code>
              </td>
              <td className="py-3">
                {a.isActive ? (
                  <Badge variant="success">已启用</Badge>
                ) : (
                  <Badge variant="default">已停用</Badge>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

const tabs: { key: Tab; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { key: 'ai', label: 'AI 配置', icon: Bot },
  { key: 'mail', label: '邮箱账户', icon: Mail },
];

export default function SettingsView() {
  const [activeTab, setActiveTab] = useState<Tab>('ai');

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">系统设置</h2>
        <p className="mt-0.5 text-sm text-gray-500 dark:text-gray-400">管理 AI 配置和邮箱账户</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-gray-200 dark:border-gray-700">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.key;
          return (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={[
                'inline-flex items-center gap-2 border-b-2 px-4 py-2.5 text-sm font-medium transition-colors',
                isActive
                  ? 'border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                  : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700 dark:text-gray-400 dark:hover:border-gray-600 dark:hover:text-gray-300',
              ].join(' ')}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab content */}
      <Card>
        {activeTab === 'ai' && <AiProfilesSection />}
        {activeTab === 'mail' && <MailAccountsSection />}
      </Card>
    </div>
  );
}
