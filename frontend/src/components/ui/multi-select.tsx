import React, { useState, useRef, useEffect } from 'react';
import { Check, X } from 'lucide-react';
import { cn, matchesCrossScript } from '../../lib/utils';
import { ComboboxOption } from './combobox';
import { useI18n } from '../../i18n';

export interface MultiSelectProps {
  options: ComboboxOption[];
  value?: string[];
  defaultValue?: string[];
  onChange?: (values: string[]) => void;
  max?: number;
  placeholder?: string;
  disabled?: boolean;
  ariaLabel?: string;
  className?: string;
}

export const MultiSelect: React.FC<MultiSelectProps> = ({
  options,
  value,
  defaultValue = [],
  onChange,
  max,
  placeholder,
  disabled = false,
  ariaLabel,
  className,
}) => {
  const { t } = useI18n();
  const [internalVal, setInternalVal] = useState<string[]>(defaultValue);
  const selected = value !== undefined ? value : internalVal;

  const [query, setQuery] = useState('');
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!open) return;
    const onOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', onOutside);
    return () => document.removeEventListener('mousedown', onOutside);
  }, [open]);

  const updateSelected = (next: string[]) => {
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  const toggleValue = (val: string) => {
    if (selected.includes(val)) {
      updateSelected(selected.filter((v) => v !== val));
    } else {
      if (max && selected.length >= max) return;
      updateSelected([...selected, val]);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Backspace' && query === '' && selected.length > 0) {
      e.preventDefault();
      updateSelected(selected.slice(0, -1));
    } else if (e.key === 'Escape') {
      setOpen(false);
    }
  };

  const filtered = options.filter((o) => matchesCrossScript(o.label, query));

  return (
    <div ref={containerRef} className={cn('relative w-full', className)}>
      <div
        onClick={() => !disabled && setOpen(true)}
        className={cn(
          'flex min-h-10 w-full flex-wrap items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1.5 text-sm focus-within:ring-2 focus-within:ring-focus',
          disabled && 'opacity-50 pointer-events-none bg-surface-muted'
        )}
      >
        {selected.map((val) => {
          const opt = options.find((o) => o.value === val);
          const label = opt?.label ?? val;
          return (
            <span
              key={val}
              className="inline-flex items-center gap-1 rounded-md bg-surface-muted border border-border px-2 py-0.5 text-xs font-medium text-primary"
            >
              <span>{label}</span>
              <button
                type="button"
                aria-label={t('common.removeChip', { label })}
                onClick={(e) => {
                  e.stopPropagation();
                  toggleValue(val);
                }}
                className="rounded text-tertiary hover:text-primary cursor-pointer"
              >
                <X className="h-3 w-3" />
              </button>
            </span>
          );
        })}
        <input
          type="text"
          value={query}
          disabled={disabled}
          aria-label={ariaLabel || placeholder || t('common.search')}
          placeholder={selected.length === 0 ? placeholder || t('common.search') : ''}
          onFocus={() => setOpen(true)}
          onChange={(e) => {
            setQuery(e.target.value);
            setOpen(true);
          }}
          onKeyDown={handleKeyDown}
          className="flex-1 min-w-24 bg-transparent text-xs sm:text-sm text-primary placeholder:text-tertiary focus:outline-none"
        />
      </div>

      {open && (
        <div
          role="listbox"
          aria-multiselectable="true"
          aria-label={ariaLabel || placeholder || t('common.search')}
          className="absolute z-40 mt-1.5 max-h-60 w-full overflow-y-auto rounded-xl border border-border bg-surface p-1.5 shadow-lg"
        >
          {filtered.length === 0 ? (
            <div className="px-3 py-3 text-center text-xs text-secondary">
              {t('common.noResults')}
            </div>
          ) : (
            filtered.map((opt) => {
              const isSelected = selected.includes(opt.value);
              return (
                <div
                  key={opt.value}
                  role="option"
                  aria-selected={isSelected}
                  onClick={() => toggleValue(opt.value)}
                  className={cn(
                    'flex items-center justify-between rounded-lg px-3 py-2 text-xs sm:text-sm cursor-pointer transition-colors hover:bg-surface-muted',
                    isSelected && 'bg-brand/10 text-brand font-medium'
                  )}
                >
                  <span>{opt.label}</span>
                  {isSelected && <Check className="h-4 w-4 text-brand" />}
                </div>
              );
            })
          )}
        </div>
      )}
    </div>
  );
};
