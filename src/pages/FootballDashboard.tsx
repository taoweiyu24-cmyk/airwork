import { Trophy, Calendar, Loader2 } from 'lucide-react';
import AppShell from '../components/layout/AppShell';
import Card from '../components/ui/Card';
import Badge from '../components/ui/Badge';

export default function FootballDashboard() {
  return (
    <AppShell title="足球数据" subtitle="比赛数据与分析">
      <div className="flex flex-col gap-6">
        {/* Heading */}
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            足球数据中心
          </h2>
          <p className="mt-0.5 text-sm text-gray-500 dark:text-gray-400">
            实时比赛数据与联赛排名
          </p>
        </div>

        {/* Recent matches */}
        <Card title="近期比赛">
          <div className="flex flex-col items-center justify-center gap-3 py-12 text-center">
            <Calendar className="h-12 w-12 text-gray-300 dark:text-gray-600" />
            <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
              暂无近期比赛数据
            </p>
            <div className="flex items-center gap-1.5">
              <Loader2 className="h-3 w-3 animate-spin text-blue-500" />
              <span className="text-xs text-blue-500">数据加载中...</span>
            </div>
          </div>
        </Card>

        {/* Team standings */}
        <Card title="球队排名">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-500 dark:text-gray-400">
                联赛排名正在同步中
              </span>
              <Badge variant="info">加载中</Badge>
            </div>

            {/* Placeholder rows */}
            <div className="flex flex-col gap-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <div
                  key={i}
                  className="flex items-center gap-3 rounded-lg bg-gray-50 p-3 dark:bg-gray-700/30"
                >
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-gray-200 text-xs font-semibold text-gray-600 dark:bg-gray-600 dark:text-gray-300">
                    {i + 1}
                  </span>
                  <div className="h-3 flex-1 animate-pulse rounded bg-gray-200 dark:bg-gray-600" />
                  <Trophy className="h-4 w-4 text-gray-300 dark:text-gray-600" />
                </div>
              ))}
            </div>

            <p className="text-center text-xs text-gray-400 dark:text-gray-500">
              数据加载中...
            </p>
          </div>
        </Card>
      </div>
    </AppShell>
  );
}
