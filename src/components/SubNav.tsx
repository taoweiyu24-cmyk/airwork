import { NavLink } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';

interface SubNavItem {
  label: string;
  to: string;
  icon: LucideIcon;
  end?: boolean;
}

interface SubNavProps {
  items: readonly SubNavItem[];
}

export default function SubNav({ items }: SubNavProps) {
  return (
    <nav className="flex gap-1 border-b border-gray-200 px-1 dark:border-gray-700">
      {items.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end={item.end}
          className={({ isActive }) =>
            [
              'flex items-center gap-1.5 border-b-2 px-3 py-2 text-xs font-medium transition-colors',
              isActive
                ? 'border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200',
            ].join(' ')
          }
        >
          <item.icon className="h-3.5 w-3.5" />
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
