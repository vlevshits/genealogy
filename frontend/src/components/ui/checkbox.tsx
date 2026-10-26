import React, { useId, useState } from 'react';
import { Check, Minus } from 'lucide-react';
import { cn } from '../../lib/utils';

export interface CheckboxProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type' | 'onChange'> {
  checked?: boolean;
  defaultChecked?: boolean;
  indeterminate?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: React.ReactNode;
  description?: React.ReactNode;
  invalid?: boolean;
  ref?: React.Ref<HTMLInputElement>;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  checked,
  defaultChecked = false,
  indeterminate = false,
  onCheckedChange,
  label,
  description,
  invalid = false,
  disabled = false,
  id,
  className,
  ref,
  ...props
}) => {
  const autoId = useId();
  const inputId = id || autoId;
  const [internalChecked, setInternalChecked] = useState(defaultChecked);
  const isChecked = checked !== undefined ? checked : internalChecked;

  const handleToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (disabled) return;
    const next = e.target.checked;
    if (checked === undefined) {
      setInternalChecked(next);
    }
    onCheckedChange?.(next);
  };

  return (
    <label
      htmlFor={inputId}
      className={cn(
        'inline-flex items-start gap-2.5 select-none cursor-pointer',
        disabled && 'opacity-50 pointer-events-none',
        className
      )}
    >
      <span className="relative flex items-center justify-center mt-0.5">
        <input
          ref={ref}
          id={inputId}
          type="checkbox"
          checked={isChecked}
          disabled={disabled}
          aria-checked={indeterminate ? 'mixed' : isChecked}
          aria-invalid={invalid || undefined}
          onChange={handleToggle}
          className="peer sr-only"
          {...props}
        />
        <span
          className={cn(
            'flex h-4.5 w-4.5 items-center justify-center rounded border transition-colors duration-150 peer-focus-visible:ring-2 peer-focus-visible:ring-focus peer-focus-visible:ring-offset-2',
            isChecked || indeterminate
              ? 'border-brand bg-brand text-surface'
              : invalid
                ? 'border-danger bg-surface'
                : 'border-border bg-surface hover:border-secondary'
          )}
        >
          {indeterminate ? (
            <Minus className="h-3 w-3 stroke-[3]" />
          ) : isChecked ? (
            <Check className="h-3 w-3 stroke-[3]" />
          ) : null}
        </span>
      </span>
      {(label || description) && (
        <span className="flex flex-col text-xs sm:text-sm leading-snug">
          {label && <span className="font-medium text-primary">{label}</span>}
          {description && <span className="text-xs text-secondary mt-0.5">{description}</span>}
        </span>
      )}
    </label>
  );
};

export interface CheckboxGroupOption {
  value: string;
  label: React.ReactNode;
  description?: React.ReactNode;
  disabled?: boolean;
}

export interface CheckboxGroupProps {
  options: CheckboxGroupOption[];
  value?: string[];
  defaultValue?: string[];
  onChange?: (values: string[]) => void;
  ariaLabel?: string;
  className?: string;
}

export const CheckboxGroup: React.FC<CheckboxGroupProps> = ({
  options,
  value,
  defaultValue = [],
  onChange,
  ariaLabel,
  className,
}) => {
  const [internalVal, setInternalVal] = useState<string[]>(defaultValue);
  const current = value !== undefined ? value : internalVal;

  const toggle = (itemVal: string, nextChecked: boolean) => {
    const next = nextChecked
      ? [...current, itemVal]
      : current.filter((v) => v !== itemVal);
    if (value === undefined) {
      setInternalVal(next);
    }
    onChange?.(next);
  };

  return (
    <div role="group" aria-label={ariaLabel} className={cn('space-y-2', className)}>
      {options.map((opt) => (
        <Checkbox
          key={opt.value}
          checked={current.includes(opt.value)}
          disabled={opt.disabled}
          label={opt.label}
          description={opt.description}
          onCheckedChange={(c) => toggle(opt.value, c)}
        />
      ))}
    </div>
  );
};
