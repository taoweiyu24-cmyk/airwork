import { Routes, Route } from 'react-router-dom';
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  FolderOpen,
  BarChart2,
} from 'lucide-react';
import AppShell from '../components/layout/AppShell';
import Card from '../components/ui/Card';
import Badge from '../components/ui/Badge';

// ── Stat cards ────────────────────────────────────────────────────────────────

interface RevenueStat {
  label: string;
  value: string;
  trend: string;
  positive: boolean;
  icon: React.ComponentType<{ className?: string }>;
}

const stats: RevenueStat[] = [
  { label: '本月收入', value: '¥128,400', trend: '+8.2%', positive: true, icon: TrendingUp },
  { label: '本月支出', value: '¥76,200', trend: '+3.1%', positive: false, icon: TrendingDown },
  { label: '净利润', value: '¥52,200', trend: '+14.5%', positive: true, icon: DollarSign },
  { label: '项目数', value: '23', trend: '+2', positive: true, icon: FolderOpen },
];

// ── Sub-pages ─────────────────────────────────────────────────────────────────

function RevenueOverview() {
  return (
    <div className="flex flex-col gap-6">
      {/* Heading */}
      <div>
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          收入概览
        </h2>
        <p className="mt-0.5 text-sm text-gray-500 dark:text-gray-400">
          当前月度财务摘要
        </p>
      </div>

      {/* Stat cards */}
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
            <span className="text-2xl font-bold text-gray-900 dark:text-white">
              {s.value}
            </span>
            <span
              className={[
                'text-xs font-medium',
                s.positive
                  ? 'text-green-600 dark:text-green-400'
                  : 'text-red-500 dark:text-red-400',
              ].join(' ')}
            >
              {s.trend} 较上月
            </span>
          </div>
        ))}
      </div>

      {/* Chart placeholder */}
      <Card title="收入趋势">
        <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
          <BarChart2 className="h-12 w-12 text-gray-300 dark:text-gray-600" />
          <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
            暂无数据
          </p>
          <p className="text-xs text-gray-400 dark:text-gray-500">
            图表功能正在开发中
          </p>
        </div>
      </Card>
    </div>
  );
}

function ProjectList() {
  return (
    <Card title="项目列表">
      <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
        <FolderOpen className="h-12 w-12 text-gray-300 dark:text-gray-600" />
        <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
          暂无项目数据
        </p>
        <Badge variant="info">数据加载中</Badge>
      </div>
    </Card>
  );
}

// ── Main export ───────────────────────────────────────────────────────────────

export default function RevenueDashboard() {
  return (
    <AppShell title="收入分析" subtitle="项目收入与成本追踪">
      <Routes>
        <Route index element={<RevenueOverview />} />
        <Route path="projects" element={<ProjectList />} />
      </Routes>
    </AppShell>
  );
}
