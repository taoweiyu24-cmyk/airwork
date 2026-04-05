import { useState } from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import {
  Wrench,
  TrendingUp,
  Trophy,
  Settings,
  ChevronLeft,
  ChevronRight,
  LayoutDashboard,
} from 'lucide-react';

interface NavItem {
  label: string;
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  end?: boolean;
}

const navItems: NavItem[] = [
  { label: '工具台', to: '/utility', icon: Wrench },
  { label: '收入分析', to: '/revenue', icon: TrendingUp },
  { label: '足球数据', to: '/football', icon: Trophy },
  { label: '设置', to: '/utility/settings', icon: Settings },
];

export default function Sidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  function isActive(to: string): boolean {
    if (to === '/utility') {
      return location.pathname === '/utility' || location.pathname.startsWith('/utility/');
    }
    if (to === '/revenue') {
      return location.pathname === '/revenue' || location.pathname.startsWith('/revenue/');
    }
    return location.pathname === to || location.pathname.startsWith(to + '/');
  }

  return (
    <aside
      className={[
        'relative flex h-full shrink-0 flex-col border-r border-gray-200 bg-white transition-all duration-200 dark:border-gray-700 dark:bg-gray-900',
        collapsed ? 'w-16' : 'w-60',
      ].join(' ')}
    >
      {/* Brand */}
      <div className="flex h-14 items-center border-b border-gray-200 px-3 dark:border-gray-700">
        <div className="flex items-center gap-2 overflow-hidden">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-blue-600 text-white">
            <LayoutDashboard className="h-4 w-4" />
          </div>
          {!collapsed && (
            <span className="truncate text-sm font-semibold text-gray-900 dark:text-white">
              WorkItemFlow
            </span>
          )}
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 p-2 py-3">
        {navItems.map((item) => {
          const active = isActive(item.to);
          return (
            <NavLink
              key={item.to}
              to={item.to}
              title={collapsed ? item.label : undefined}
              className={[
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                collapsed ? 'justify-center' : '',
                active
                  ? 'bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400'
                  : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-white',
              ].join(' ')}
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {!collapsed && <span className="truncate">{item.label}</span>}
            </NavLink>
          );
        })}
      </nav>

      {/* Collapse toggle */}
      <button
        onClick={() => setCollapsed((c) => !c)}
        className="absolute -right-3 top-[3.75rem] z-10 flex h-6 w-6 items-center justify-center rounded-full border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-50 hover:text-gray-700 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200"
        aria-label={collapsed ? '展开侧栏' : '收起侧栏'}
      >
        {collapsed ? (
          <ChevronRight className="h-3 w-3" />
        ) : (
          <ChevronLeft className="h-3 w-3" />
        )}
      </button>
    </aside>
  );
}
