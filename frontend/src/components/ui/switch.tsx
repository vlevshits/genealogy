import React, { useId, useState } from 'react';
import { cn } from '../../lib/utils';

export interface SwitchProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onChange'> {
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: React.ReactNode;
  description?: React.ReactNode;
  ref?: React.Ref<HTMLButtonElement>;
}

export const Switch: React.FC<SwitchProps> = ({
  checked,
  defaultChecked = false,
  onCheckedChange,
  label,
  description,
  disabled = false,
  id,
  className,
  ref,
  ...props
}) => {
  const autoId = useId();
  const switchId = id || autoId;
  const [internalChecked, setInternalChecked] = useState(defaultChecked);
  const isChecked = checked !== undefined ? checked : internalChecked;

  const handleToggle = () => {
    if (disabled) return;
    const next = !isChecked;
    if (checked === undefined) {
      setInternalChecked(next);
    }
    onCheckedChange?.(next);
  };

  return (
    <div className={cn('flex items-start justify-between gap-3', className)}>
      {(label || description) && (
        <label
          htmlFor={switchId}
          className={cn('flex flex-col cursor-pointer select-none', disabled && 'opacity-50')}
        >
          {label && <span className="text-sm font-medium text-primary">{label}</span>}
          {description && <span className="text-xs text-secondary mt-0.5">{description}</span>}
        </label>
      )}
      <button
        ref={ref}
        id={switchId}
        type="button"
        role="switch"
        aria-checked={isChecked}
        aria-label={typeof label === 'string' ? label : props['aria-label']}
        disabled={disabled}
        onClick={handleToggle}
        className={cn(
          'relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none touch-target-expand',
          isChecked ? 'bg-brand' : 'bg-surface-muted border-border'
        )}
        {...props}
      >
        <span
          className={cn(
            'pointer-events-none inline-block h-5 w-5 transform rounded-full bg-surface shadow-sm transition-transform duration-200',
            isChecked ? 'translate-x-5' : 'translate-x-0'
          )}
        />
      </button>
    </div>
  );
};
