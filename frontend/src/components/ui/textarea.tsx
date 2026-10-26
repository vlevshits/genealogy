import React, { useState } from 'react';
import { cn } from '../../lib/utils';

export interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  invalid?: boolean;
  showCount?: boolean;
  minRows?: number;
  maxRows?: number;
  ref?: React.Ref<HTMLTextAreaElement>;
}

export const Textarea: React.FC<TextareaProps> = ({
  invalid = false,
  readOnly = false,
  disabled = false,
  maxLength,
  showCount = false,
  minRows = 3,
  maxRows = 12,
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

  const lineCount = currentVal ? currentVal.split('\n').length : minRows;
  const computedRows = Math.min(maxRows, Math.max(minRows, lineCount));

  return (
    <div className="relative w-full">
      <textarea
        ref={ref}
        rows={props.rows ?? computedRows}
        value={value}
        defaultValue={defaultValue}
        disabled={disabled}
        readOnly={readOnly}
        maxLength={maxLength}
        aria-invalid={invalid || undefined}
        onChange={(e) => {
          if (value === undefined) setInternalVal(e.target.value);
          onChange?.(e);
        }}
        className={cn(
          'w-full rounded-lg border bg-surface px-3 py-2 text-sm text-primary placeholder:text-tertiary transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          invalid
            ? 'border-danger focus-visible:ring-danger'
            : 'border-border hover:border-secondary/50',
          readOnly && 'bg-surface-muted text-secondary cursor-default',
          disabled && 'opacity-50 pointer-events-none bg-surface-muted',
          className
        )}
        {...props}
      />
      {maxLength && showCount && (
        <div className="mt-1 flex justify-end font-mono text-[11px] text-tertiary">
          {currentVal.length}/{maxLength}
        </div>
      )}
    </div>
  );
};
