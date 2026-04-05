import { useState, useEffect, useCallback } from 'react';
import { Routes, Route } from 'react-router-dom';
import {
  ClipboardList,
  Mail,
  Plus,
  RefreshCw,
  Loader2,
  Trash2,
  Pencil,
  AlertCircle,
} from 'lucide-react';
import AppShell from '../components/layout/AppShell';
import Card from '../components/ui/Card';
import Button from '../components/ui/Button';
import Badge from '../components/ui/Badge';
import type { BadgeProps } from '../components/ui/Badge';
import WorkItemForm from '../components/WorkItemForm';
import DashboardOverview from '../components/DashboardOverview';
import SettingsView from '../components/SettingsView';
import GisViewComponent from '../components/GisView';
import type { WorkItem, WorkItemStatus, Priority, Source } from '../types/domain';
import * as api from '../services/api';

// ── Label / badge maps ──────────────────────────────────────────────────────

type LabelMap<K extends string> = Record<K, { label: string; variant: BadgeProps['variant'] }>;

const statusMap: LabelMap<WorkItemStatus> = {
  new: { label: '新建', variant: 'warning' },
  inProgress: { label: '进行中', variant: 'info' },
  reviewing: { label: '审核中', variant: 'info' },
  done: { label: '已完成', variant: 'success' },
  archived: { label: '已归档', variant: 'default' },
};

const priorityMap: LabelMap<Priority> = {
  low: { label: '低', variant: 'default' },
  normal: { label: '普通', variant: 'info' },
  high: { label: '高', variant: 'warning' },
  urgent: { label: '紧急', variant: 'danger' },
};

const sourceLabel: Record<Source, string> = {
  manual: '手动', email: '邮件', file: '文件', api: 'API',
};

function formatDate(epoch: number): string {
  return new Date(epoch * 1000).toLocaleDateString('zh-CN');
}

function WorkItemsList() {
  const [items, setItems] = useState<WorkItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<WorkItem | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const loadItems = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.listWorkItems();
      setItems(data);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '加载工作项失败';
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  const handleCreate = useCallback(() => {
    setEditingItem(null);
    setModalOpen(true);
  }, []);

  const handleEdit = useCallback((item: WorkItem) => {
    setEditingItem(item);
    setModalOpen(true);
  }, []);

  const handleFormSubmit = useCallback(
    async (item: WorkItem) => {
      if (editingItem) {
        await api.updateWorkItem(item);
      } else {
        await api.createWorkItem(item);
      }
      await loadItems();
    },
    [editingItem, loadItems],
  );

  const handleDelete = useCallback(
    async (id: string) => {
      setDeletingId(id);
      try {
        await api.deleteWorkItem(id);
        await loadItems();
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : '删除失败';
        setError(msg);
      } finally {
        setDeletingId(null);
      }
    },
    [loadItems],
  );

  const confirmDelete = useCallback(
    (item: WorkItem) => {
      if (window.confirm(`确定要删除「${item.title}」吗？`)) {
        handleDelete(item.id);
      }
    },
    [handleDelete],
  );

  if (loading) {
    return (
      <Card title="工作项列表">
        <div className="flex items-center justify-center gap-2 py-16">
          <Loader2 className="h-5 w-5 animate-spin text-gray-400" />
          <span className="text-sm text-gray-500 dark:text-gray-400">加载中...</span>
        </div>
      </Card>
    );
  }

  if (error) {
    return (
      <Card title="工作项列表">
        <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
          <AlertCircle className="h-10 w-10 text-red-400" />
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
          <Button variant="secondary" size="sm" onClick={loadItems}>
            <RefreshCw className="h-4 w-4" />
            重试
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <>
      <Card title="工作项列表">
        {/* Toolbar */}
        <div className="mb-4 flex items-center justify-between">
          <span className="text-sm text-gray-500 dark:text-gray-400">
            共 {items.length} 条
          </span>
          <div className="flex gap-2">
            <Button variant="secondary" size="sm" onClick={loadItems}>
              <RefreshCw className="h-4 w-4" />
              刷新
            </Button>
            <Button variant="primary" size="sm" onClick={handleCreate}>
              <Plus className="h-4 w-4" />
              新建工作项
            </Button>
          </div>
        </div>

        {/* Empty state */}
        {items.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
            <ClipboardList className="h-12 w-12 text-gray-300 dark:text-gray-600" />
            <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
              暂无工作项数据
            </p>
            <p className="text-xs text-gray-400 dark:text-gray-500">
              点击"新建工作项"开始创建
            </p>
          </div>
        ) : (
          /* Table */
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-gray-200 dark:border-gray-700">
                  {['标题', '状态', '优先级', '来源', '创建日期', '操作'].map((h) => (
                    <th key={h} className="pb-3 pr-4 font-medium text-gray-500 last:pr-0 dark:text-gray-400">
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr
                    key={item.id}
                    className="border-b border-gray-100 last:border-b-0 hover:bg-gray-50 dark:border-gray-800 dark:hover:bg-gray-800/50"
                  >
                    <td className="py-3 pr-4">
                      <button
                        className="text-left font-medium text-gray-900 hover:text-blue-600 dark:text-white dark:hover:text-blue-400"
                        onClick={() => handleEdit(item)}
                      >
                        {item.title}
                      </button>
                      {item.tags.length > 0 && (
                        <div className="mt-1 flex flex-wrap gap-1">
                          {item.tags.map((tag) => (
                            <span
                              key={tag}
                              className="inline-block rounded bg-gray-100 px-1.5 py-0.5 text-xs text-gray-500 dark:bg-gray-700 dark:text-gray-400"
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      )}
                    </td>
                    <td className="py-3 pr-4">
                      <Badge variant={statusMap[item.status].variant}>
                        {statusMap[item.status].label}
                      </Badge>
                    </td>
                    <td className="py-3 pr-4">
                      <Badge variant={priorityMap[item.priority].variant}>
                        {priorityMap[item.priority].label}
                      </Badge>
                    </td>
                    <td className="py-3 pr-4 text-gray-600 dark:text-gray-400">
                      {sourceLabel[item.source]}
                    </td>
                    <td className="py-3 pr-4 text-gray-600 dark:text-gray-400">
                      {formatDate(item.createdAt)}
                    </td>
                    <td className="py-3">
                      <div className="flex gap-1">
                        <button
                          className="rounded-lg p-1.5 text-gray-400 hover:bg-gray-100 hover:text-blue-600 dark:hover:bg-gray-700 dark:hover:text-blue-400"
                          onClick={() => handleEdit(item)}
                          title="编辑"
                        >
                          <Pencil className="h-4 w-4" />
                        </button>
                        <button
                          className="rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 disabled:opacity-50 dark:hover:bg-red-900/20 dark:hover:text-red-400"
                          onClick={() => confirmDelete(item)}
                          disabled={deletingId === item.id}
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
      </Card>

      <WorkItemForm
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        onSubmit={handleFormSubmit}
        editingItem={editingItem}
      />
    </>
  );
}

function Inbox() {
  return (
    <Card title="邮件收件箱">
      <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
        <Mail className="h-12 w-12 text-gray-300 dark:text-gray-600" />
        <p className="text-sm font-medium text-gray-500 dark:text-gray-400">收件箱为空</p>
        <Button variant="secondary" size="sm">
          <RefreshCw className="h-4 w-4" />
          同步邮件
        </Button>
      </div>
    </Card>
  );
}


export default function UtilityDashboard() {
  return (
    <AppShell title="工具台" subtitle="工作项管理与分析">
      <Routes>
        <Route index element={<DashboardOverview />} />
        <Route path="items" element={<WorkItemsList />} />
        <Route path="inbox" element={<Inbox />} />
        <Route path="gis" element={<GisViewComponent />} />
        <Route path="settings" element={<SettingsView />} />
      </Routes>
    </AppShell>
  );
}
