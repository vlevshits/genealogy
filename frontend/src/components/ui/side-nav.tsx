import React from 'react';
import { cn } from '../../lib/utils';
import { Select } from './select';

export interface SideNavItem {
  value: string;
  label: string;
  icon?: React.ReactNode;
  badge?: string | number;
}

export interface SideNavProps {
  items: SideNavItem[];
  value: string;
  onChange: (value: string) => void;
  ariaLabel?: string;
  className?: string;
}

export const SideNav: React.FC<SideNavProps> = ({
  items,
  value,
  onChange,
  ariaLabel = 'Section navigation',
  className,
}) => {
  return (
    <div className={cn('w-full', className)}>
      <div className="md:hidden">
        <Select
          ariaLabel={ariaLabel}
          value={value}
          onChange={onChange}
          options={items.map((i) => ({ value: i.value, label: i.label, icon: i.icon }))}
        />
      </div>
      <nav aria-label={ariaLabel} className="hidden md:flex md:flex-col md:gap-1">
        {items.map((item) => {
          const active = item.value === value;
          return (
            <button
              key={item.value}
              type="button"
              aria-current={active ? 'page' : undefined}
              onClick={() => onChange(item.value)}
              className={cn(
                'flex items-center justify-between gap-2.5 rounded-xl px-3 py-2 text-left text-xs sm:text-sm font-medium transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                active
                  ? 'bg-brand/10 text-brand font-semibold'
                  : 'text-secondary hover:bg-surface-muted hover:text-primary'
              )}
            >
              <span className="flex items-center gap-2.5 min-w-0">
                <span className="shrink-0">{item.icon}</span>
                <span className="whitespace-normal text-left leading-snug">{item.label}</span>
              </span>
              {item.badge !== undefined && (
                <span className="rounded-md bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] text-secondary shrink-0">
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>
    </div>
  );
};
