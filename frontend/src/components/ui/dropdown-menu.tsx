import React, { useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
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

const VIEWPORT_PADDING = 8;

export function clampMenuHorizontalOffset(
  menuRect: Pick<DOMRect, 'left' | 'right' | 'width'>,
  viewportWidth: number,
  margin: number = VIEWPORT_PADDING
): number {
  if (viewportWidth <= 0) return 0;
  let dx = 0;
  if (menuRect.right > viewportWidth - margin) {
    dx = viewportWidth - margin - menuRect.right;
  }
  if (menuRect.left + dx < margin) {
    dx += margin - (menuRect.left + dx);
  }
  return Math.round(dx);
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
  const activeIndexRef = useRef(0);
  activeIndexRef.current = activeIndex;
  const [horizontalShift, setHorizontalShift] = useState(0);
  const menuId = useId();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const triggerWrapperRef = useRef<HTMLDivElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const itemRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const pendingFocusModeRef = useRef<'first' | 'last'>('first');

  const findEnabledIndex = useCallback(
    (startIndex: number, direction: 1 | -1): number => {
      if (items.length === 0) return -1;
      for (let step = 0; step < items.length; step += 1) {
        const candidate =
          (startIndex + direction * step + items.length) % items.length;
        if (!items[candidate]?.disabled) {
          return candidate;
        }
      }
      return -1;
    },
    [items]
  );

  const focusItemAt = useCallback((index: number) => {
    if (index < 0 || index >= items.length) return;
    activeIndexRef.current = index;
    setActiveIndex(index);
    itemRefs.current[index]?.focus();
  }, [items.length]);

  const restoreTriggerFocus = useCallback(() => {
    const focusTarget =
      triggerWrapperRef.current?.querySelector<HTMLElement>(
        'button, a[href], [tabindex]:not([tabindex="-1"])'
      ) ?? triggerWrapperRef.current;
    focusTarget?.focus();
  }, []);

  useLayoutEffect(() => {
    if (!open) {
      setHorizontalShift(0);
      return;
    }

    const updatePosition = () => {
      const menuEl = menuRef.current;
      if (!menuEl || typeof window === 'undefined') return;
      menuEl.style.transform = '';
      const rect = menuEl.getBoundingClientRect();
      const shift = clampMenuHorizontalOffset(rect, window.innerWidth);
      setHorizontalShift(shift);
    };

    updatePosition();
    window.addEventListener('resize', updatePosition);
    return () => window.removeEventListener('resize', updatePosition);
  }, [open, align, items.length]);

  useEffect(() => {
    if (!open) return;

    const targetIndex =
      pendingFocusModeRef.current === 'last'
        ? findEnabledIndex(items.length - 1, -1)
        : findEnabledIndex(0, 1);
    if (targetIndex !== -1) {
      activeIndexRef.current = targetIndex;
      setActiveIndex(targetIndex);
      itemRefs.current[targetIndex]?.focus();
    }

    const onOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        setOpen(false);
        restoreTriggerFocus();
      } else if (e.key === 'Tab') {
        setOpen(false);
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        const next = findEnabledIndex(activeIndexRef.current + 1, 1);
        if (next !== -1) focusItemAt(next);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        const prev = findEnabledIndex(activeIndexRef.current - 1, -1);
        if (prev !== -1) focusItemAt(prev);
      } else if (e.key === 'Home') {
        e.preventDefault();
        const first = findEnabledIndex(0, 1);
        if (first !== -1) focusItemAt(first);
      } else if (e.key === 'End') {
        e.preventDefault();
        const last = findEnabledIndex(items.length - 1, -1);
        if (last !== -1) focusItemAt(last);
      }
    };
    document.addEventListener('mousedown', onOutside);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onOutside);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [open, items.length, findEnabledIndex, focusItemAt, restoreTriggerFocus]);

  const handleTriggerKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (open) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      pendingFocusModeRef.current = 'first';
      setOpen(true);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      pendingFocusModeRef.current = 'last';
      setOpen(true);
    }
  };

  const enhancedTrigger = React.isValidElement<Record<string, unknown>>(trigger)
    ? React.cloneElement(trigger, {
        'aria-haspopup': 'menu',
        'aria-expanded': open,
        'aria-controls': open ? menuId : undefined,
      })
    : trigger;

  return (
    <div ref={containerRef} className={cn('relative inline-block', className)}>
      <div
        ref={triggerWrapperRef}
        onClick={() => {
          pendingFocusModeRef.current = 'first';
          setOpen((o) => !o);
        }}
        onKeyDown={handleTriggerKeyDown}
        className="inline-flex cursor-pointer"
      >
        {enhancedTrigger}
      </div>

      {open && (
        <div
          ref={menuRef}
          id={menuId}
          role="menu"
          aria-label={ariaLabel}
          style={
            horizontalShift !== 0
              ? { transform: `translateX(${horizontalShift}px)` }
              : undefined
          }
          className={cn(
            'absolute z-40 mt-1.5 min-w-52 max-w-[calc(100vw-1rem)] rounded-xl border border-border bg-surface p-1.5 shadow-lg',
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
                ref={(el) => {
                  itemRefs.current[idx] = el;
                }}
                type="button"
                role={item.checked !== undefined ? 'menuitemcheckbox' : 'menuitem'}
                aria-checked={item.checked}
                disabled={item.disabled}
                tabIndex={idx === activeIndex ? 0 : -1}
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
                  'flex w-full min-h-10 items-center justify-between gap-3 rounded-lg px-2.5 py-2 text-xs sm:text-sm text-left transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                  item.danger
                    ? 'text-danger hover:bg-danger/10 focus-visible:bg-danger/10'
                    : idx === activeIndex
                      ? 'bg-surface-muted text-primary'
                      : 'text-primary hover:bg-surface-muted',
                  item.disabled && 'opacity-40 cursor-not-allowed'
                )}
              >
                <span className="flex items-center gap-2 min-w-0">
                  {item.checked !== undefined && (
                    <span className="flex h-4 w-4 shrink-0 items-center justify-center">
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
