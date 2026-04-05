import { useState, useEffect, useMemo } from 'react';
import { ClipboardList, Plus, RefreshCw, MapPin, CheckCircle2, Clock, Loader2, Hash, AlertCircle } from 'lucide-react';
import Card from './ui/Card';
import Button from './ui/Button';
import Badge from './ui/Badge';
import type { BadgeProps } from './ui/Badge';
import * as api from '../services/api';
import type { WorkItem, WorkItemStatus, Priority } from '../types/domain';

// ── Lookup tables ────────────────────────────────────────────────────────────

type BadgeVariant = NonNullable<BadgeProps['variant']>;
type LabelAndVariant = { label: string; variant: BadgeVariant };

const STATUS_META: Record<WorkItemStatus, LabelAndVariant> = {
  new: { label: '待处理', variant: 'warning' },
  inProgress: { label: '进行中', variant: 'info' },
  reviewing: { label: '审核中', variant: 'info' },
  done: { label: '已完成', variant: 'success' },
  archived: { label: '已归档', variant: 'default' },
};

const PRIORITY_META: Record<Priority, LabelAndVariant> = {
  low: { label: '低', variant: 'default' },
  normal: { label: '普通', variant: 'info' },
  high: { label: '高', variant: 'warning' },
  urgent: { label: '紧急', variant: 'danger' },
};

interface StatItem {
  label: string; value: number;
  badge: { label: string; variant: BadgeVariant };
  icon: React.ComponentType<{ className?: string }>;
}

function relativeTime(epochMs: number): string {
  const diff = Math.floor((Date.now() - epochMs) / 1000);
  if (diff < 60) return '刚刚';
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 2592000) return `${Math.floor(diff / 86400)} 天前`;
  return `${Math.floor(diff / 2592000)} 个月前`;
}

function buildStats(items: readonly WorkItem[]): StatItem[] {
  const pending = items.filter((i) => i.status === 'new').length;
  const active = items.filter((i) => i.status === 'inProgress' || i.status === 'reviewing').length;
  const completed = items.filter((i) => i.status === 'done' || i.status === 'archived').length;
  return [
    { label: '待处理', value: pending, badge: { label: '待处理', variant: 'warning' }, icon: Clock },
    { label: '进行中', value: active, badge: { label: '进行中', variant: 'info' }, icon: Loader2 },
    { label: '已完成', value: completed, badge: { label: '已完成', variant: 'success' }, icon: CheckCircle2 },
    { label: '总计', value: items.length, badge: { label: '全部', variant: 'default' }, icon: Hash },
  ];
}

// ── Component ────────────────────────────────────────────────────────────────
export default function DashboardOverview() {
  const [items, setItems] = useState<WorkItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await api.listWorkItems();
        if (!cancelled) setItems(data);
      } catch (err: unknown) {
        if (!cancelled) setError(err instanceof Error ? err.message : '加载工作项失败');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const stats = useMemo(() => buildStats(items), [items]);
  const recent = useMemo(
    () => [...items].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 5),
    [items],
  );

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-20">
        <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
        <p className="text-sm text-gray-500 dark:text-gray-400">正在加载数据...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-20">
        <AlertCircle className="h-8 w-8 text-red-400" />
        <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        <Button variant="secondary" size="sm" onClick={() => window.location.reload()}>
          重试
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      {/* Welcome */}
      <div>
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          你好，欢迎回来！
        </h2>
        <p className="mt-0.5 text-sm text-gray-500 dark:text-gray-400">
          以下是当前工作项的概览
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((s) => (
          <div
            key={s.label}
            className="flex flex-col gap-2 rounded-xl border border-gray-200 bg-white p-5 dark:border-gray-700 dark:bg-gray-800"
          >
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500 dark:text-gray-400">{s.label}</span>
              <s.icon className="h-4 w-4 text-gray-400 dark:text-gray-500" />
            </div>
            <div className="flex items-end justify-between">
              <span className="text-3xl font-bold text-gray-900 dark:text-white">
                {s.value}
              </span>
              <Badge variant={s.badge.variant}>{s.badge.label}</Badge>
            </div>
          </div>
        ))}
      </div>

      {/* Recent work items */}
      <Card title="近期工作项">
        {recent.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 py-10 text-center">
            <ClipboardList className="h-10 w-10 text-gray-300 dark:text-gray-600" />
            <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
              暂无工作项
            </p>
            <p className="text-xs text-gray-400 dark:text-gray-500">
              点击下方"新建工作项"开始创建
            </p>
          </div>
        ) : (
          <ul className="divide-y divide-gray-100 dark:divide-gray-700">
            {recent.map((item) => (
              <li
                key={item.id}
                className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0"
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-gray-900 dark:text-white">
                    {item.title}
                  </p>
                  <p className="mt-0.5 text-xs text-gray-400 dark:text-gray-500">
                    {relativeTime(item.updatedAt)}
                  </p>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <Badge variant={PRIORITY_META[item.priority].variant}>
                    {PRIORITY_META[item.priority].label}
                  </Badge>
                  <Badge variant={STATUS_META[item.status].variant}>
                    {STATUS_META[item.status].label}
                  </Badge>
                </div>
              </li>
            ))}
          </ul>
        )}
      </Card>

      {/* Quick actions */}
      <Card title="快捷操作">
        <div className="flex flex-wrap gap-3">
          <Button variant="primary" size="sm">
            <Plus className="h-4 w-4" />
            新建工作项
          </Button>
          <Button variant="secondary" size="sm">
            <RefreshCw className="h-4 w-4" />
            同步邮件
          </Button>
          <Button variant="secondary" size="sm">
            <MapPin className="h-4 w-4" />
            空间分析
          </Button>
        </div>
      </Card>
    </div>
  );
}
