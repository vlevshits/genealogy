import React, { useState } from 'react';
import { cn } from '../../lib/utils';

export interface RadioOption<T extends string = string> {
  value: T;
  label: React.ReactNode;
  description?: React.ReactNode;
  icon?: React.ReactNode;
  disabled?: boolean;
}

export interface RadioGroupProps<T extends string = string> {
  options: RadioOption<T>[];
  value?: T;
  defaultValue?: T;
  onChange?: (value: T) => void;
  name?: string;
  ariaLabel?: string;
  disabled?: boolean;
  className?: string;
  ref?: React.Ref<HTMLDivElement>;
}

export function RadioGroup<T extends string = string>({
  options,
  value,
  defaultValue,
  onChange,
  ariaLabel,
  disabled = false,
  className,
  ref,
}: RadioGroupProps<T>) {
  const [internalVal, setInternalVal] = useState<T | undefined>(
    defaultValue ?? options[0]?.value
  );
  const current = value !== undefined ? value : internalVal;

  const select = (next: T) => {
    if (disabled) return;
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  return (
    <div
      ref={ref}
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn('space-y-2', disabled && 'opacity-50 pointer-events-none', className)}
    >
      {options.map((opt) => {
        const checked = current === opt.value;
        return (
          <button
            key={opt.value}
            type="button"
            role="radio"
            aria-checked={checked}
            disabled={disabled || opt.disabled}
            onClick={() => select(opt.value)}
            className={cn(
              'flex w-full items-start gap-2.5 text-left cursor-pointer rounded-lg p-1.5 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
              opt.disabled && 'opacity-50 pointer-events-none'
            )}
          >
            <span
              className={cn(
                'mt-0.5 flex h-4.5 w-4.5 shrink-0 items-center justify-center rounded-full border transition-colors',
                checked ? 'border-brand bg-brand' : 'border-border bg-surface'
              )}
            >
              {checked && <span className="h-2 w-2 rounded-full bg-surface" />}
            </span>
            <span className="flex flex-col text-xs sm:text-sm">
              <span className="font-medium text-primary">{opt.label}</span>
              {opt.description && (
                <span className="text-xs text-secondary mt-0.5">{opt.description}</span>
              )}
            </span>
          </button>
        );
      })}
    </div>
  );
}

export function RadioCardGroup<T extends string = string>({
  options,
  value,
  defaultValue,
  onChange,
  ariaLabel,
  disabled = false,
  className,
  ref,
}: RadioGroupProps<T>) {
  const [internalVal, setInternalVal] = useState<T | undefined>(
    defaultValue ?? options[0]?.value
  );
  const current = value !== undefined ? value : internalVal;

  const select = (next: T) => {
    if (disabled) return;
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  return (
    <div
      ref={ref}
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn(
        'grid grid-cols-1 sm:grid-cols-2 gap-3',
        disabled && 'opacity-50 pointer-events-none',
        className
      )}
    >
      {options.map((opt) => {
        const checked = current === opt.value;
        return (
          <button
            key={opt.value}
            type="button"
            role="radio"
            aria-checked={checked}
            disabled={disabled || opt.disabled}
            onClick={() => select(opt.value)}
            className={cn(
              'flex items-start gap-3 rounded-xl border p-4 text-left transition-all duration-150 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
              checked
                ? 'border-brand bg-brand/5 ring-1 ring-brand'
                : 'border-border bg-surface hover:bg-surface-muted/60',
              opt.disabled && 'opacity-50 pointer-events-none'
            )}
          >
            {opt.icon && (
              <div
                className={cn(
                  'flex h-9 w-9 shrink-0 items-center justify-center rounded-lg',
                  checked ? 'bg-brand text-surface' : 'bg-surface-muted text-secondary'
                )}
              >
                {opt.icon}
              </div>
            )}
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-sm font-semibold text-primary">{opt.label}</span>
                <span
                  className={cn(
                    'flex h-4 w-4 shrink-0 items-center justify-center rounded-full border',
                    checked ? 'border-brand bg-brand' : 'border-border bg-surface'
                  )}
                >
                  {checked && <span className="h-1.5 w-1.5 rounded-full bg-surface" />}
                </span>
              </div>
              {opt.description && (
                <p className="mt-1 text-xs text-secondary leading-relaxed">{opt.description}</p>
              )}
            </div>
          </button>
        );
      })}
    </div>
  );
}
