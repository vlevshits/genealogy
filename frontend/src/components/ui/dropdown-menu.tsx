import React, { useState, useRef, useEffect } from 'react';
import { Check } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Kbd } from './kbd';

export interface DropdownMenuItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  disabledReason?: string;
  checked?: boolean;
  separatorBefore?: boolean;
  groupLabel?: string;
  onSelect?: () => void;
}

export interface DropdownMenuProps {
  trigger: React.ReactNode;
  items: DropdownMenuItem[];
  align?: 'start' | 'end';
  ariaLabel?: string;
  className?: string;
}

export const DropdownMenu: React.FC<DropdownMenuProps> = ({
  trigger,
  items,
  align = 'end',
  ariaLabel,
  className,
}) => {
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!open) return;
    const onOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOpen(false);
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        setActiveIndex((i) => (i + 1) % Math.max(1, items.length));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setActiveIndex((i) => (i - 1 + Math.max(1, items.length)) % Math.max(1, items.length));
      }
    };
    document.addEventListener('mousedown', onOutside);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onOutside);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [open, items.length]);

  return (
    <div ref={containerRef} className={cn('relative inline-block', className)}>
      <div
        onClick={() => setOpen((o) => !o)}
        className="inline-flex cursor-pointer"
      >
        {trigger}
      </div>

      {open && (
        <div
          role="menu"
          aria-label={ariaLabel}
          className={cn(
            'absolute z-40 mt-1.5 min-w-52 rounded-xl border border-border bg-surface p-1.5 shadow-lg',
            align === 'end' ? 'right-0' : 'left-0'
          )}
        >
          {items.map((item, idx) => (
            <React.Fragment key={item.id}>
              {item.separatorBefore && <div className="my-1 border-t border-border" />}
              {item.groupLabel && (
                <div className="px-2.5 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
                  {item.groupLabel}
                </div>
              )}
              <button
                type="button"
                role={item.checked !== undefined ? 'menuitemcheckbox' : 'menuitem'}
                aria-checked={item.checked}
                disabled={item.disabled}
                title={item.disabled ? item.disabledReason : undefined}
                onMouseEnter={() => setActiveIndex(idx)}
                onClick={() => {
                  if (item.disabled) return;
                  item.onSelect?.();
                  if (item.checked === undefined) {
                    setOpen(false);
                  }
                }}
                className={cn(
                  'flex w-full items-center justify-between gap-3 rounded-lg px-2.5 py-2 text-xs sm:text-sm text-left transition-colors cursor-pointer',
                  item.danger
                    ? 'text-danger hover:bg-danger/10'
                    : idx === activeIndex
                      ? 'bg-surface-muted text-primary'
                      : 'text-primary hover:bg-surface-muted',
                  item.disabled && 'opacity-40 cursor-not-allowed'
                )}
              >
                <span className="flex items-center gap-2 min-w-0">
                  {item.checked !== undefined && (
                    <span className="flex h-4 w-4 items-center justify-center">
                      {item.checked && <Check className="h-3.5 w-3.5 text-brand" />}
                    </span>
                  )}
                  {item.icon}
                  <span className="truncate">{item.label}</span>
                </span>
                {item.shortcut && <Kbd>{item.shortcut}</Kbd>}
              </button>
            </React.Fragment>
          ))}
        </div>
      )}
    </div>
  );
};
