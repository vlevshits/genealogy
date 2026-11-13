import React, { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { Check, ChevronDown, Plus, RotateCcw, Search, X } from 'lucide-react';
import { cn, matchesCrossScript } from '../../lib/utils';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { Spinner } from './spinner';
import { useI18n } from '../../i18n';
import { useFloatingMenu } from './use-floating-menu';

export interface ComboboxOption<T = unknown> {
  value: string;
  label: string;
  description?: string;
  data?: T;
}

export interface ComboboxProps<T = unknown> {
  options?: ComboboxOption<T>[];
  loadOptions?: (query: string, signal: AbortSignal) => Promise<ComboboxOption<T>[]>;
  value?: string | null;
  defaultValue?: string | null;
  onChange?: (value: string | null, option?: ComboboxOption<T>) => void;
  renderOption?: (option: ComboboxOption<T>, isSelected: boolean) => React.ReactNode;
  onCreate?: (query: string) => void;
  recent?: ComboboxOption<T>[];
  allowFreeText?: boolean;
  clearable?: boolean;
  minChars?: number;
  placeholder?: string;
  disabled?: boolean;
  invalid?: boolean;
  ariaLabel?: string;
  className?: string;
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
  ref?: React.Ref<HTMLInputElement>;
}

export function Combobox<T = unknown>({
  options = [],
  loadOptions,
  value,
  defaultValue = null,
  onChange,
  renderOption,
  onCreate,
  recent = [],
  allowFreeText = false,
  clearable = true,
  minChars = 0,
  placeholder,
  disabled = false,
  invalid = false,
  ariaLabel,
  className,
  hasMore = false,
  loadingMore = false,
  onLoadMore,
  ref,
}: ComboboxProps<T>) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [internalVal, setInternalVal] = useState<string | null>(defaultValue);
  const currentValue = value !== undefined ? value : internalVal;

  const allStatic = [...options, ...recent];
  const selectedOpt = allStatic.find((o) => o.value === currentValue);

  const [query, setQuery] = useState<string>(selectedOpt?.label ?? (allowFreeText && currentValue ? currentValue : ''));
  const [asyncItems, setAsyncItems] = useState<ComboboxOption<T>[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeIndex, setActiveIndex] = useState(0);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const menuStyle = useFloatingMenu(containerRef, open);
  const abortRef = useRef<AbortController | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const found = [...options, ...recent, ...asyncItems].find((o) => o.value === currentValue);
    if (found) {
      setQuery(found.label);
    } else if (allowFreeText && currentValue !== null && currentValue !== undefined) {
      setQuery(currentValue);
    } else if (!currentValue) {
      setQuery('');
    }
  }, [currentValue]);

  useEffect(() => {
    if (!open) return;
    const handleOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        containerRef.current &&
        !containerRef.current.contains(target) &&
        !menuRef.current?.contains(target)
      ) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleOutside);
    return () => document.removeEventListener('mousedown', handleOutside);
  }, [open]);

  const runAsyncFetch = (q: string) => {
    if (!loadOptions) return;
    if (q.trim().length < minChars) {
      setAsyncItems([]);
      setLoading(false);
      return;
    }
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (abortRef.current) abortRef.current.abort();

    setLoading(true);
    setError(null);

    debounceRef.current = setTimeout(async () => {
      const controller = new AbortController();
      abortRef.current = controller;
      try {
        const res = await loadOptions(q, controller.signal);
        if (!controller.signal.aborted) {
          setAsyncItems(res);
          setLoading(false);
          setActiveIndex(0);
        }
      } catch (err) {
        if (!controller.signal.aborted) {
          setError(getLocalizedErrorMessage(err, t));
          setLoading(false);
        }
      }
    }, 150);
  };

  const filterQuery = selectedOpt && query === selectedOpt.label ? '' : query;

  const displayedOptions: ComboboxOption<T>[] = loadOptions
    ? query.trim() === ''
      ? recent.length > 0
        ? recent
        : options
      : asyncItems.length > 0
        ? asyncItems
        : options.filter((o) =>
            matchesCrossScript(`${o.label} ${o.description ?? ''}`, query)
          )
    : filterQuery.trim() === '' && recent.length > 0
      ? recent
      : options.filter((o) =>
          matchesCrossScript(`${o.label} ${o.description ?? ''}`, filterQuery)
        );

  const selectOption = (opt: ComboboxOption<T>) => {
    if (value === undefined) setInternalVal(opt.value);
    setQuery(opt.label);
    onChange?.(opt.value, opt);
    setOpen(false);
  };

  const handleClear = () => {
    if (value === undefined) setInternalVal(null);
    setQuery('');
    setAsyncItems([]);
    onChange?.(null, undefined);
    if (loadOptions && minChars === 0) {
      runAsyncFetch('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!open && (e.key === 'ArrowDown' || e.key === 'Enter')) {
      setOpen(true);
      if (loadOptions) runAsyncFetch(query);
      return;
    }
    if (e.key === 'Escape') {
      setOpen(false);
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIndex((i) => Math.min(displayedOptions.length - 1, i + 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIndex((i) => Math.max(0, i - 1));
    } else if (e.key === 'Home') {
      setActiveIndex(0);
    } else if (e.key === 'End') {
      setActiveIndex(Math.max(0, displayedOptions.length - 1));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const chosen = displayedOptions[activeIndex];
      if (chosen) {
        selectOption(chosen);
      } else if (onCreate && query.trim()) {
        onCreate(query.trim());
        setOpen(false);
      } else if (allowFreeText) {
        onChange?.(query.trim());
        setOpen(false);
      }
    }
  };

  return (
    <div ref={containerRef} className={cn('relative w-full', className)}>
      <div className="relative flex items-center">
        <Search className="pointer-events-none absolute left-3 h-4 w-4 text-tertiary" />
        <input
          ref={ref}
          type="text"
          role="combobox"
          aria-expanded={open}
          aria-autocomplete="list"
          aria-label={ariaLabel || placeholder || t('common.search')}
          aria-invalid={invalid || undefined}
          disabled={disabled}
          value={query}
          placeholder={placeholder || t('common.search')}
          onFocus={(e) => {
            setOpen(true);
            e.currentTarget.select();
            if (loadOptions) runAsyncFetch(query);
          }}
          onChange={(e) => {
            const next = e.target.value;
            setQuery(next);
            setOpen(true);
            if (allowFreeText) {
              if (value === undefined) setInternalVal(next);
              onChange?.(next);
            }
            if (loadOptions) {
              runAsyncFetch(next);
            }
          }}
          onKeyDown={handleKeyDown}
          className={cn(
            'min-h-11 w-full rounded-lg border bg-surface pl-9 pr-14 py-2 text-sm text-primary placeholder:text-tertiary transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
            invalid ? 'border-danger' : 'border-border hover:border-secondary/50',
            disabled && 'opacity-50 pointer-events-none bg-surface-muted'
          )}
        />
        <div className="absolute right-2 flex items-center gap-1">
          {clearable && (query || currentValue) && !disabled && (
            <button
              type="button"
              aria-label={t('common.clear')}
              onClick={handleClear}
              className="rounded p-1 text-tertiary hover:bg-surface-muted hover:text-primary cursor-pointer"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
          <ChevronDown className="h-4 w-4 text-tertiary pointer-events-none" />
        </div>
      </div>

      {open && menuStyle && createPortal(
        <div
          ref={menuRef}
          style={menuStyle}
          role="listbox"
          aria-label={ariaLabel || placeholder || t('common.search')}
          onScroll={(e) => {
            const el = e.currentTarget;
            if (
              hasMore &&
              !loading &&
              !loadingMore &&
              el.scrollHeight - el.scrollTop <= el.clientHeight + 48
            ) {
              onLoadMore?.();
            }
          }}
          className="z-[70] overflow-y-auto rounded-xl border border-border bg-surface p-1.5 shadow-lg"
        >
          {query.trim() === '' && recent.length > 0 && (
            <div className="px-2.5 py-1 text-[11px] font-semibold uppercase tracking-wider text-tertiary">
              {t('common.recentHeading')}
            </div>
          )}

          {loading && (
            <div className="flex items-center justify-center gap-2 py-6 text-xs text-secondary">
              <Spinner size="sm" />
              <span>{t('common.loading')}</span>
            </div>
          )}

          {!loading && error && (
            <div className="flex items-center justify-between gap-2 rounded-lg bg-danger/10 px-3 py-2.5 text-xs text-danger">
              <span>{error}</span>
              <button
                type="button"
                onClick={() => runAsyncFetch(query)}
                className="inline-flex items-center gap-1 font-semibold underline cursor-pointer"
              >
                <RotateCcw className="h-3 w-3" />
                {t('common.retry')}
              </button>
            </div>
          )}

          {!loading && !error && displayedOptions.length === 0 && (
            <div className="px-3 py-4 text-center text-xs text-secondary">
              {t('common.noResults')}
            </div>
          )}

          {!loading &&
            !error &&
            displayedOptions.map((opt, idx) => {
              const isSelected = opt.value === currentValue;
              return (
                <div
                  key={opt.value}
                  role="option"
                  aria-selected={isSelected}
                  onMouseEnter={() => setActiveIndex(idx)}
                  onClick={() => selectOption(opt)}
                  className={cn(
                    'flex items-center justify-between gap-2 rounded-lg px-3 py-2 text-xs sm:text-sm cursor-pointer transition-colors',
                    idx === activeIndex ? 'bg-surface-muted text-primary' : 'text-primary',
                    isSelected && 'bg-brand/10 text-brand font-medium'
                  )}
                >
                  {renderOption ? (
                    <div className="flex-1 min-w-0">{renderOption(opt, isSelected)}</div>
                  ) : (
                    <div className="min-w-0 flex-1">
                      <div className="truncate">{opt.label}</div>
                      {opt.description && (
                        <div className="truncate text-[11px] text-secondary">
                          {opt.description}
                        </div>
                      )}
                    </div>
                  )}
                  {isSelected && <Check className="h-4 w-4 shrink-0 text-brand" />}
                </div>
              );
            })}

          {loadingMore && (
            <div className="flex items-center justify-center gap-2 py-2.5 text-xs text-secondary">
              <Spinner size="sm" />
              <span>{t('common.loading')}</span>
            </div>
          )}

          {onCreate && query.trim().length > 0 && (
            <button
              type="button"
              onClick={() => {
                onCreate(query.trim());
                setOpen(false);
              }}
              className="mt-1 flex w-full items-center gap-2 rounded-lg border-t border-border px-3 py-2 text-left text-xs font-medium text-brand hover:bg-surface-muted cursor-pointer"
            >
              <Plus className="h-3.5 w-3.5" />
              <span>{t('common.createOption', { query: query.trim() })}</span>
            </button>
          )}
        </div>,
        document.body
      )}
    </div>
  );
}
