import React, { useState, useEffect, useRef } from 'react';
import { Eye, EyeOff, Search, X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { ControlSize } from '../../types/common';
import { useI18n } from '../../i18n';

export interface TextInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  size?: ControlSize;
  leftIcon?: React.ReactNode;
  rightSlot?: React.ReactNode;
  invalid?: boolean;
  showCount?: boolean;
  ref?: React.Ref<HTMLInputElement>;
}

export const TextInput: React.FC<TextInputProps> = ({
  size = 'md',
  leftIcon,
  rightSlot,
  invalid = false,
  readOnly = false,
  disabled = false,
  maxLength,
  showCount = false,
  value,
  defaultValue,
  onChange,
  className,
  ref,
  ...props
}) => {
  const [internalVal, setInternalVal] = useState<string>(
    defaultValue !== undefined ? String(defaultValue) : ''
  );
  const currentVal = value !== undefined ? String(value) : internalVal;

  const sizeClasses: Record<ControlSize, string> = {
    sm: 'min-h-8 text-xs px-2.5 py-1 touch-target-expand',
    md: 'min-h-11 text-sm px-3 py-2',
    lg: 'min-h-12 text-base px-4 py-2.5',
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (value === undefined) {
      setInternalVal(e.target.value);
    }
    onChange?.(e);
  };

  return (
    <div className="relative flex items-center w-full">
      {leftIcon && (
        <span className="pointer-events-none absolute left-3 flex items-center text-tertiary">
          {leftIcon}
        </span>
      )}
      <input
        ref={ref}
        value={value}
        defaultValue={defaultValue}
        onChange={handleChange}
        disabled={disabled}
        readOnly={readOnly}
        maxLength={maxLength}
        aria-invalid={invalid || undefined}
        className={cn(
          'w-full rounded-lg border bg-surface text-primary placeholder:text-tertiary transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          invalid
            ? 'border-danger focus-visible:ring-danger'
            : 'border-border hover:border-secondary/50',
          readOnly && 'bg-surface-muted text-secondary cursor-default',
          disabled && 'opacity-50 pointer-events-none bg-surface-muted',
          sizeClasses[size],
          Boolean(leftIcon) && 'pl-9',
          Boolean(rightSlot || (maxLength && showCount)) && 'pr-10',
          className
        )}
        {...props}
      />
      {(rightSlot || (maxLength && showCount)) && (
        <div className="absolute right-2.5 flex items-center gap-1.5 text-xs text-tertiary">
          {maxLength && showCount && (
            <span className="font-mono text-[11px]">
              {currentVal.length}/{maxLength}
            </span>
          )}
          {rightSlot}
        </div>
      )}
    </div>
  );
};

export interface SearchInputProps extends Omit<TextInputProps, 'leftIcon'> {
  onSearch?: (query: string) => void;
  debounceMs?: number;
}

export const SearchInput: React.FC<SearchInputProps> = ({
  value,
  defaultValue,
  onChange,
  onSearch,
  debounceMs = 200,
  placeholder,
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const [internalVal, setInternalVal] = useState<string>(
    defaultValue !== undefined ? String(defaultValue) : ''
  );
  const currentVal = value !== undefined ? String(value) : internalVal;
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const triggerSearch = (nextVal: string) => {
    if (!onSearch) return;
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      onSearch(nextVal);
    }, debounceMs);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const next = e.target.value;
    if (value === undefined) {
      setInternalVal(next);
    }
    onChange?.(e);
    triggerSearch(next);
  };

  const handleClear = () => {
    if (value === undefined) {
      setInternalVal('');
    }
    const syntheticEvent = {
      target: { value: '' },
    } as React.ChangeEvent<HTMLInputElement>;
    onChange?.(syntheticEvent);
    onSearch?.('');
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Escape' && currentVal) {
      e.preventDefault();
      handleClear();
    }
    props.onKeyDown?.(e);
  };

  return (
    <TextInput
      ref={ref}
      type="search"
      value={currentVal}
      onChange={handleChange}
      onKeyDown={handleKeyDown}
      placeholder={placeholder ?? t('common.search')}
      leftIcon={<Search className="h-4 w-4" />}
      rightSlot={
        currentVal ? (
          <button
            type="button"
            aria-label={t('common.clear')}
            onClick={handleClear}
            className="inline-flex min-h-10 min-w-10 items-center justify-center rounded-lg -mr-1.5 text-tertiary hover:bg-surface-muted hover:text-primary cursor-pointer"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        ) : undefined
      }
      {...props}
    />
  );
};

export interface PasswordInputProps extends Omit<TextInputProps, 'type'> {
  showStrengthHint?: boolean;
}

export const PasswordInput: React.FC<PasswordInputProps> = ({
  showStrengthHint = false,
  value,
  defaultValue,
  onChange,
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const [visible, setVisible] = useState(false);
  const [internalVal, setInternalVal] = useState<string>(
    defaultValue !== undefined ? String(defaultValue) : ''
  );
  const currentVal = value !== undefined ? String(value) : internalVal;

  return (
    <div className="w-full space-y-1">
      <TextInput
        ref={ref}
        type={visible ? 'text' : 'password'}
        value={value}
        defaultValue={defaultValue}
        onChange={(e) => {
          if (value === undefined) setInternalVal(e.target.value);
          onChange?.(e);
        }}
        rightSlot={
          <button
            type="button"
            aria-label={visible ? t('common.hidePassword') : t('common.showPassword')}
            onClick={() => setVisible((v) => !v)}
            className="inline-flex min-h-10 min-w-10 items-center justify-center rounded-lg -mr-1.5 text-tertiary hover:bg-surface-muted hover:text-primary cursor-pointer"
          >
            {visible ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        }
        {...props}
      />
      {showStrengthHint && (
        <div className="flex items-center justify-between text-[11px] text-secondary">
          <span>{t('common.passwordStrengthHint')}</span>
          <span
            className={cn(
              'font-mono font-medium',
              currentVal.length >= 10 ? 'text-success' : 'text-tertiary'
            )}
          >
            {currentVal.length}/10+
          </span>
        </div>
      )}
    </div>
  );
};

export const EmailInput: React.FC<Omit<TextInputProps, 'type'>> = ({ ref, ...props }) => {
  return <TextInput ref={ref} type="email" autoComplete="email" {...props} />;
};
