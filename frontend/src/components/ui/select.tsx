import React, { useState, useRef, useEffect } from 'react';
import { Check, ChevronDown, X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { ControlSize } from '../../types/common';
import { useI18n } from '../../i18n';

export interface SelectOption<T extends string = string> {
  value: T;
  label: string;
  description?: string;
  icon?: React.ReactNode;
  disabled?: boolean;
  group?: string;
}

export interface SelectProps<T extends string = string> {
  options: SelectOption<T>[];
  value?: T;
  defaultValue?: T;
  onChange?: (value: T) => void;
  placeholder?: string;
  size?: ControlSize;
  invalid?: boolean;
  disabled?: boolean;
  ariaLabel?: string;
  'aria-label'?: string;
  id?: string;
  className?: string;
  ref?: React.Ref<HTMLButtonElement>;
}

export function Select<T extends string = string>({
  options,
  value,
  defaultValue,
  onChange,
  placeholder,
  size = 'md',
  invalid = false,
  disabled = false,
  ariaLabel,
  'aria-label': ariaLabelAttr,
  id,
  className,
  ref,
}: SelectProps<T>) {
  const { t } = useI18n();
  const resolvedAriaLabel = ariaLabel || ariaLabelAttr;
  const [open, setOpen] = useState(false);
  const [internalVal, setInternalVal] = useState<T | undefined>(defaultValue);
  const [activeIndex, setActiveIndex] = useState<number>(0);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const currentValue = value !== undefined ? value : internalVal;
  const selectedOption = options.find((o) => o.value === currentValue);

  useEffect(() => {
    if (!open) return;
    const handleOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleOutside);
    return () => document.removeEventListener('mousedown', handleOutside);
  }, [open]);

  const handleSelect = (opt: SelectOption<T>) => {
    if (opt.disabled) return;
    if (value === undefined) {
      setInternalVal(opt.value);
    }
    onChange?.(opt.value);
    setOpen(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;
    const enabled = options.filter((o) => !o.disabled);
    if (!open) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        setOpen(true);
      }
      return;
    }

    if (e.key === 'Escape') {
      e.preventDefault();
      setOpen(false);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIndex((prev) => (prev + 1) % Math.max(1, enabled.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIndex((prev) => (prev - 1 + Math.max(1, enabled.length)) % Math.max(1, enabled.length));
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const target = enabled[activeIndex];
      if (target) handleSelect(target);
    }
  };

  const sizeClasses: Record<ControlSize, string> = {
    sm: 'min-h-8 px-2.5 py-1 text-xs touch-target-expand',
    md: 'min-h-11 px-3 py-2 text-sm',
    lg: 'min-h-12 px-4 py-2.5 text-base',
  };

  return (
    <div ref={containerRef} className={cn('relative w-full', className)}>
      <button
        ref={ref}
        id={id}
        type="button"
        role="combobox"
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={resolvedAriaLabel || placeholder}
        aria-invalid={invalid || undefined}
        disabled={disabled}
        onClick={() => setOpen((o) => !o)}
        onKeyDown={handleKeyDown}
        className={cn(
          'flex w-full items-center justify-between gap-2 rounded-lg border bg-surface text-left transition-colors duration-150 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          invalid ? 'border-danger' : 'border-border hover:border-secondary/50',
          disabled && 'opacity-50 pointer-events-none bg-surface-muted',
          sizeClasses[size]
        )}
      >
        <span className="flex items-center gap-2 truncate">
          {selectedOption?.icon}
          <span className={cn('truncate', selectedOption ? 'text-primary' : 'text-tertiary')}>
            {selectedOption ? selectedOption.label : placeholder || t('common.search')}
          </span>
        </span>
        <ChevronDown
          className={cn(
            'h-4 w-4 shrink-0 text-tertiary transition-transform duration-150',
            open && 'rotate-180'
          )}
        />
      </button>

      {open && (
        <>
          {/* Mobile bottom-sheet backdrop (< 768px) */}
          <div
            className="fixed inset-0 z-40 bg-primary/30 backdrop-blur-xs md:hidden"
            onClick={() => setOpen(false)}
          />
          <div
            role="listbox"
            aria-label={resolvedAriaLabel || placeholder}
            className={cn(
              'z-50 overflow-y-auto border border-border bg-surface p-1.5 shadow-lg',
              'fixed inset-x-0 bottom-0 max-h-[70vh] rounded-t-2xl md:absolute md:inset-x-auto md:bottom-auto md:mt-1.5 md:w-full md:max-h-64 md:rounded-xl'
            )}
          >
            <div className="flex items-center justify-between border-b border-border px-3 py-2 md:hidden">
              <span className="text-xs font-semibold text-secondary">
                {placeholder || resolvedAriaLabel}
              </span>
              <button
                type="button"
                aria-label={t('common.close')}
                onClick={() => setOpen(false)}
                className="rounded p-1 text-secondary"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            {options.map((opt, idx) => {
              const isSelected = opt.value === currentValue;
              return (
                <div
                  key={opt.value}
                  role="option"
                  aria-selected={isSelected}
                  aria-disabled={opt.disabled}
                  onClick={() => handleSelect(opt)}
                  onMouseEnter={() => setActiveIndex(idx)}
                  className={cn(
                    'flex items-center justify-between gap-2 rounded-lg px-3 py-2 text-xs sm:text-sm cursor-pointer transition-colors',
                    isSelected
                      ? 'bg-brand/10 text-brand font-medium'
                      : idx === activeIndex
                        ? 'bg-surface-muted text-primary'
                        : 'text-primary hover:bg-surface-muted',
                    opt.disabled && 'opacity-40 pointer-events-none'
                  )}
                >
                  <div className="flex items-center gap-2 min-w-0">
                    {opt.icon}
                    <div className="truncate">
                      <div>{opt.label}</div>
                      {opt.description && (
                        <div className="text-[11px] text-secondary truncate">
                          {opt.description}
                        </div>
                      )}
                    </div>
                  </div>
                  {isSelected && <Check className="h-4 w-4 shrink-0 text-brand" />}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
