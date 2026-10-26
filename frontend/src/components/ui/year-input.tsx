import React, { useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface YearInputProps {
  value?: number | null;
  defaultValue?: number | null;
  min?: number;
  max?: number;
  onChange?: (year: number | null) => void;
  disabled?: boolean;
  invalid?: boolean;
  ariaLabel?: string;
  className?: string;
}

export const YearInput: React.FC<YearInputProps> = ({
  value,
  defaultValue = 1900,
  min = 1000,
  max = new Date().getFullYear(),
  onChange,
  disabled = false,
  invalid = false,
  ariaLabel,
  className,
}) => {
  const { t } = useI18n();
  const [internalVal, setInternalVal] = useState<number | null>(defaultValue);
  const current = value !== undefined ? value : internalVal;

  const setYear = (next: number | null) => {
    const clamped =
      next === null || Number.isNaN(next) ? null : Math.max(min, Math.min(max, next));
    if (value === undefined) setInternalVal(clamped);
    onChange?.(clamped);
  };

  return (
    <div className={cn('inline-flex items-center rounded-lg border border-border bg-surface', invalid && 'border-danger', className)}>
      <input
        type="number"
        min={min}
        max={max}
        disabled={disabled}
        aria-label={ariaLabel || t('gendate.yearLabel')}
        value={current ?? ''}
        onChange={(e) => {
          const raw = e.target.value;
          setYear(raw === '' ? null : parseInt(raw, 10));
        }}
        className="w-20 bg-transparent px-3 py-2 font-mono text-sm text-primary focus:outline-none"
      />
      <div className="flex flex-col border-l border-border">
        <button
          type="button"
          aria-label="Increment year"
          disabled={disabled}
          onClick={() => setYear((current ?? 1900) + 1)}
          className="px-1.5 py-0.5 text-tertiary hover:bg-surface-muted hover:text-primary cursor-pointer"
        >
          <ChevronUp className="h-3 w-3" />
        </button>
        <button
          type="button"
          aria-label="Decrement year"
          disabled={disabled}
          onClick={() => setYear((current ?? 1900) - 1)}
          className="border-t border-border px-1.5 py-0.5 text-tertiary hover:bg-surface-muted hover:text-primary cursor-pointer"
        >
          <ChevronDown className="h-3 w-3" />
        </button>
      </div>
    </div>
  );
};
