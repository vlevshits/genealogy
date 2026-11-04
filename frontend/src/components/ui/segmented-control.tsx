import React, { useState, useRef } from 'react';
import { cn } from '../../lib/utils';
import { ControlSize } from '../../types/common';

export interface SegmentedOption<T extends string = string> {
  value: T;
  label: React.ReactNode;
  icon?: React.ReactNode;
  disabled?: boolean;
  accentClassName?: string;
}

export interface SegmentedControlProps<T extends string = string> {
  options: SegmentedOption<T>[];
  value?: T;
  defaultValue?: T;
  onChange?: (value: T) => void;
  size?: ControlSize;
  disabled?: boolean;
  ariaLabel?: string;
  'aria-label'?: string;
  className?: string;
  ref?: React.Ref<HTMLDivElement>;
}

export function SegmentedControl<T extends string = string>({
  options,
  value,
  defaultValue,
  onChange,
  size = 'md',
  disabled = false,
  ariaLabel,
  'aria-label': ariaLabelAttr,
  className,
  ref,
}: SegmentedControlProps<T>) {
  const resolvedAriaLabel = ariaLabel || ariaLabelAttr;
  const [internalValue, setInternalValue] = useState<T | undefined>(
    defaultValue ?? options[0]?.value
  );
  const currentValue = value !== undefined ? value : internalValue;
  const itemRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const selectValue = (nextValue: T) => {
    if (disabled) return;
    if (value === undefined) {
      setInternalValue(nextValue);
    }
    onChange?.(nextValue);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    const enabledIndices = options
      .map((opt, idx) => (!opt.disabled ? idx : -1))
      .filter((idx) => idx !== -1);
    const pos = enabledIndices.indexOf(index);
    if (pos === -1) return;

    let nextPos = -1;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      nextPos = (pos + 1) % enabledIndices.length;
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      nextPos = (pos - 1 + enabledIndices.length) % enabledIndices.length;
    } else if (e.key === 'Home') {
      e.preventDefault();
      nextPos = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      nextPos = enabledIndices.length - 1;
    }

    if (nextPos !== -1) {
      const targetIdx = enabledIndices[nextPos];
      const opt = options[targetIdx];
      selectValue(opt.value);
      itemRefs.current[targetIdx]?.focus();
    }
  };

  const sizeClasses: Record<ControlSize, string> = {
    sm: 'min-h-7 px-2.5 py-1 text-xs',
    md: 'min-h-9 px-3 py-1.5 text-xs sm:text-sm',
    lg: 'min-h-11 px-4 py-2 text-sm',
  };

  return (
    <div
      ref={ref}
      role="radiogroup"
      aria-label={resolvedAriaLabel}
      className={cn(
        'inline-flex items-center gap-0.5 rounded-lg border border-border bg-surface-muted p-0.5',
        disabled && 'opacity-50 pointer-events-none',
        className
      )}
    >
      {options.map((opt, idx) => {
        const isSelected = currentValue === opt.value;
        return (
          <button
            key={opt.value}
            ref={(el) => {
              itemRefs.current[idx] = el;
            }}
            type="button"
            role="radio"
            aria-checked={isSelected}
            disabled={disabled || opt.disabled}
            tabIndex={isSelected ? 0 : -1}
            onClick={() => selectValue(opt.value)}
            onKeyDown={(e) => handleKeyDown(e, idx)}
            className={cn(
              'inline-flex items-center justify-center gap-1.5 rounded-md font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus cursor-pointer text-center leading-tight h-full [&>svg]:shrink-0',
              sizeClasses[size],
              isSelected
                ? cn('bg-surface text-primary shadow-xs', opt.accentClassName)
                : 'text-secondary hover:text-primary',
              opt.disabled && 'opacity-40 pointer-events-none'
            )}
          >
            {opt.icon}
            <span>{opt.label}</span>
          </button>
        );
      })}
    </div>
  );
}
