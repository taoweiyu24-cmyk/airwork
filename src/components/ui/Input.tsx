import type { InputHTMLAttributes } from 'react';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

export default function Input({ label, id, className = '', ...props }: InputProps) {
  const inputId = id ?? label;

  return (
    <div className="flex flex-col gap-1">
      {label && (
        <label
          htmlFor={inputId}
          className="text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          {label}
        </label>
      )}
      <input
        id={inputId}
        className={[
          'rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 placeholder-gray-400',
          'focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/20',
          'disabled:cursor-not-allowed disabled:bg-gray-50 disabled:text-gray-500',
          'dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:placeholder-gray-500',
          'dark:focus:border-blue-400 dark:focus:ring-blue-400/20',
          className,
        ].join(' ')}
        {...props}
      />
    </div>
  );
}
