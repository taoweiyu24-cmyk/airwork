import type { ReactNode } from 'react';

export interface CardProps {
  title?: string;
  children: ReactNode;
  className?: string;
}

export default function Card({ title, children, className = '' }: CardProps) {
  return (
    <div
      className={[
        'rounded-xl border border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800',
        className,
      ].join(' ')}
    >
      {title && (
        <div className="border-b border-gray-200 px-5 py-4 dark:border-gray-700">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-white">
            {title}
          </h3>
        </div>
      )}
      <div className="p-5">{children}</div>
    </div>
  );
}
