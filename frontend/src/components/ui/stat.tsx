import React from 'react';
import { cn } from '../../lib/utils';

export interface StatProps extends React.HTMLAttributes<HTMLDivElement> {
  label: string;
  value: string | number;
  hint?: string;
  icon?: React.ReactNode;
  href?: string;
  ref?: React.Ref<HTMLDivElement>;
}

export const Stat: React.FC<StatProps> = ({
  label,
  value,
  hint,
  icon,
  href,
  className,
  ref,
  ...props
}) => {
  const inner = (
    <>
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-medium text-secondary">{label}</span>
        {icon && <span className="text-brand">{icon}</span>}
      </div>
      <div className="mt-2 font-serif text-2xl sm:text-3xl font-bold tracking-tight text-primary">
        {typeof value === 'number' ? value.toLocaleString() : value}
      </div>
      {hint && <div className="mt-1 text-xs text-tertiary">{hint}</div>}
    </>
  );

  if (href) {
    return (
      <a
        href={href}
        className={cn(
          'block rounded-2xl border border-border bg-surface p-4 shadow-xs transition-all hover:shadow-md hover:border-brand/40',
          className
        )}
      >
        {inner}
      </a>
    );
  }

  return (
    <div
      ref={ref}
      className={cn('rounded-2xl border border-border bg-surface p-4 shadow-xs', className)}
      {...props}
    >
      {inner}
    </div>
  );
};
