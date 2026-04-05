import { useState, useEffect, useCallback } from 'react';
import { Bot, Mail, Loader2, Sparkles, ServerCrash, Plus, Pencil, Trash2 } from 'lucide-react';
import Card from './ui/Card';
import Badge from './ui/Badge';
import Button from './ui/Button';
import Modal from './ui/Modal';
import AiProfileForm from './settings/AiProfileForm';
import MailAccountForm from './settings/MailAccountForm';
import {
  listAiProfiles,
  createAiProfile,
  updateAiProfile,
  deleteAiProfile,
  listMailAccounts,
  createMailAccount,
  updateMailAccount,
  deleteMailAccount,
} from '../services/api';
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

// ── Delete Confirmation ──────────────────────────────────────────────────────

interface DeleteConfirmProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  itemName: string;
  deleting: boolean;
}

function DeleteConfirm({ isOpen, onClose, onConfirm, itemName, deleting }: DeleteConfirmProps) {
  return (
    <Modal isOpen={isOpen} onClose={onClose} title="确认删除">
      <div className="flex flex-col gap-4">
        <p className="text-sm text-gray-600 dark:text-gray-300">
          确定要删除 <strong>{itemName}</strong> 吗？此操作不可撤销。
        </p>
        <div className="flex justify-end gap-3">
          <Button variant="secondary" onClick={onClose} disabled={deleting}>
            取消
          </Button>
          <Button variant="danger" onClick={onConfirm} disabled={deleting}>
            {deleting ? '删除中...' : '删除'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

// ── AI Profiles Section ──────────────────────────────────────────────────────

function AiProfilesSection() {
  const [profiles, setProfiles] = useState<AiProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<AiProfile | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AiProfile | null>(null);
  const [deleting, setDeleting] = useState(false);

  const fetchProfiles = useCallback(() => {
    setLoading(true);
    setError(null);
    listAiProfiles()
      .then(setProfiles)
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : '加载 AI 配置失败';
        setError(msg);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { fetchProfiles(); }, [fetchProfiles]);

  const handleCreate = async (profile: AiProfile) => {
    await createAiProfile(profile);
    fetchProfiles();
  };

  const handleUpdate = async (profile: AiProfile) => {
    await updateAiProfile(profile);
    fetchProfiles();
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await deleteAiProfile(deleteTarget.id);
      setDeleteTarget(null);
      fetchProfiles();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '删除失败';
      setError(msg);
    } finally {
      setDeleting(false);
    }
  };

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };

  const openEdit = (p: AiProfile) => {
    setEditing(p);
    setFormOpen(true);
  };

  return (
    <>
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">AI 配置列表</h3>
        <Button size="sm" onClick={openCreate}>
          <Plus className="h-4 w-4" />
          新建
        </Button>
      </div>

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={fetchProfiles} />}
      {!loading && !error && profiles.length === 0 && (
        <EmptyState icon={Sparkles} message="暂无 AI 配置" />
      )}
      {!loading && !error && profiles.length > 0 && (
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">名称</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">提供商</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">模型</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">API 密钥</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">状态</th>
                <th className="pb-3 font-medium text-gray-500 dark:text-gray-400">操作</th>
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
                  <td className="py-3 pr-4">
                    {p.isDefault ? (
                      <Badge variant="success">默认</Badge>
                    ) : (
                      <Badge variant="default">可用</Badge>
                    )}
                  </td>
                  <td className="py-3">
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => openEdit(p)}
                        className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-blue-600 dark:hover:bg-gray-700 dark:hover:text-blue-400"
                        title="编辑"
                      >
                        <Pencil className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => setDeleteTarget(p)}
                        className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-red-600 dark:hover:bg-gray-700 dark:hover:text-red-400"
                        title="删除"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {formOpen && (
        <AiProfileForm
          isOpen={formOpen}
          onClose={() => setFormOpen(false)}
          onSubmit={editing ? handleUpdate : handleCreate}
          initial={editing}
        />
      )}

      <DeleteConfirm
        isOpen={deleteTarget !== null}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
        itemName={deleteTarget?.name ?? ''}
        deleting={deleting}
      />
    </>
  );
}

// ── Mail Accounts Section ────────────────────────────────────────────────────

function MailAccountsSection() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<MailAccount | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<MailAccount | null>(null);
  const [deleting, setDeleting] = useState(false);

  const fetchAccounts = useCallback(() => {
    setLoading(true);
    setError(null);
    listMailAccounts()
      .then(setAccounts)
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : '加载邮箱账户失败';
        setError(msg);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { fetchAccounts(); }, [fetchAccounts]);

  const handleCreate = async (account: MailAccount) => {
    await createMailAccount(account);
    fetchAccounts();
  };

  const handleUpdate = async (account: MailAccount) => {
    await updateMailAccount(account);
    fetchAccounts();
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await deleteMailAccount(deleteTarget.id);
      setDeleteTarget(null);
      fetchAccounts();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '删除失败';
      setError(msg);
    } finally {
      setDeleting(false);
    }
  };

  const openCreate = () => {
    setEditing(null);
    setFormOpen(true);
  };

  const openEdit = (a: MailAccount) => {
    setEditing(a);
    setFormOpen(true);
  };

  return (
    <>
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">邮箱账户列表</h3>
        <Button size="sm" onClick={openCreate}>
          <Plus className="h-4 w-4" />
          新建
        </Button>
      </div>

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={fetchAccounts} />}
      {!loading && !error && accounts.length === 0 && (
        <EmptyState icon={Mail} message="暂无邮箱账户" />
      )}
      {!loading && !error && accounts.length > 0 && (
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">名称</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">邮箱</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">提供商</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">IMAP 主机</th>
                <th className="pb-3 pr-4 font-medium text-gray-500 dark:text-gray-400">状态</th>
                <th className="pb-3 font-medium text-gray-500 dark:text-gray-400">操作</th>
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
                  <td className="py-3 pr-4">
                    {a.isActive ? (
                      <Badge variant="success">已启用</Badge>
                    ) : (
                      <Badge variant="default">已停用</Badge>
                    )}
                  </td>
                  <td className="py-3">
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => openEdit(a)}
                        className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-blue-600 dark:hover:bg-gray-700 dark:hover:text-blue-400"
                        title="编辑"
                      >
                        <Pencil className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => setDeleteTarget(a)}
                        className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-red-600 dark:hover:bg-gray-700 dark:hover:text-red-400"
                        title="删除"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {formOpen && (
        <MailAccountForm
          isOpen={formOpen}
          onClose={() => setFormOpen(false)}
          onSubmit={editing ? handleUpdate : handleCreate}
          initial={editing}
        />
      )}

      <DeleteConfirm
        isOpen={deleteTarget !== null}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
        itemName={deleteTarget?.name ?? ''}
        deleting={deleting}
      />
    </>
  );
}

// ── Tabs ─────────────────────────────────────────────────────────────────────

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
