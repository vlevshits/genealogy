import React, { useState, useEffect } from 'react';
import { cn } from '../../lib/utils';
import { Badge } from './badge';

export interface TabItem<T extends string = string> {
  value: T;
  label: string;
  count?: number;
  icon?: React.ReactNode;
  disabled?: boolean;
  content?: React.ReactNode;
}

export interface TabsProps<T extends string = string> {
  items: TabItem<T>[];
  value?: T;
  defaultValue?: T;
  onChange?: (value: T) => void;
  urlParam?: string;
  ariaLabel?: string;
  className?: string;
}

export function Tabs<T extends string = string>({
  items,
  value,
  defaultValue,
  onChange,
  urlParam,
  ariaLabel,
  className,
}: TabsProps<T>) {
  const [internalVal, setInternalVal] = useState<T>(() => {
    if (urlParam && typeof window !== 'undefined') {
      const sp = new URLSearchParams(window.location.search);
      const fromUrl = sp.get(urlParam) as T | null;
      if (fromUrl && items.some((i) => i.value === fromUrl)) {
        return fromUrl;
      }
    }
    return defaultValue ?? items[0]?.value;
  });

  const activeValue = value !== undefined ? value : internalVal;

  useEffect(() => {
    if (!urlParam || typeof window === 'undefined') return;
    const url = new URL(window.location.href);
    url.searchParams.set(urlParam, activeValue);
    window.history.replaceState({}, '', url.toString());
  }, [activeValue, urlParam]);

  const selectTab = (next: T) => {
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  const activeTab = items.find((i) => i.value === activeValue);

  return (
    <div className={cn('w-full space-y-4', className)}>
      <div
        role="tablist"
        aria-label={ariaLabel}
        className="flex items-center gap-1 overflow-x-auto border-b border-border"
      >
        {items.map((item) => {
          const isSelected = item.value === activeValue;
          return (
            <button
              key={item.value}
              type="button"
              role="tab"
              aria-selected={isSelected}
              disabled={item.disabled}
              onClick={() => selectTab(item.value)}
              className={cn(
                'inline-flex items-center gap-2 whitespace-nowrap border-b-2 px-3.5 py-2.5 text-xs sm:text-sm font-medium transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                isSelected
                  ? 'border-brand text-brand font-semibold'
                  : 'border-transparent text-secondary hover:border-border hover:text-primary',
                item.disabled && 'opacity-40 pointer-events-none'
              )}
            >
              {item.icon}
              <span>{item.label}</span>
              {item.count !== undefined && (
                <Badge variant={isSelected ? 'brand' : 'neutral'} size="sm">
                  {item.count}
                </Badge>
              )}
            </button>
          );
        })}
      </div>
      {activeTab?.content && (
        <div role="tabpanel" className="pt-1">
          {activeTab.content}
        </div>
      )}
    </div>
  );
}
