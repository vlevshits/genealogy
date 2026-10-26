import React, { useId } from 'react';
import { AlertCircle, AlertTriangle } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useFormContext } from './form';
import { useI18n } from '../../i18n';

export interface FormFieldProps {
  name: string;
  label: React.ReactNode;
  required?: boolean;
  optionalText?: boolean;
  hint?: React.ReactNode;
  error?: string;
  warning?: string;
  layout?: 'vertical' | 'horizontal';
  children:
    | React.ReactElement
    | ((fieldProps: {
        id: string;
        value: unknown;
        onChange: (val: unknown) => void;
        invalid: boolean;
        'aria-describedby'?: string;
        'aria-invalid'?: boolean;
      }) => React.ReactNode);
  className?: string;
}

export const FormField: React.FC<FormFieldProps> = ({
  name,
  label,
  required = false,
  optionalText = false,
  hint,
  error: propError,
  warning: propWarning,
  layout = 'vertical',
  children,
  className,
}) => {
  const { t } = useI18n();
  const form = useFormContext();
  const baseId = useId();
  const fieldId = `field-${name}-${baseId}`;
  const hintId = `${fieldId}-hint`;
  const errorId = `${fieldId}-error`;
  const warningId = `${fieldId}-warning`;

  const resolvedError = propError ?? form?.errors[name];
  const resolvedWarning = propWarning ?? form?.warnings[name];
  const isInvalid = Boolean(resolvedError);

  const describedBy = [
    hint ? hintId : null,
    resolvedError ? errorId : null,
    resolvedWarning ? warningId : null,
  ]
    .filter(Boolean)
    .join(' ') || undefined;

  const renderControl = () => {
    if (typeof children === 'function') {
      return children({
        id: fieldId,
        value: form?.values[name],
        onChange: (val: unknown) => form?.setValue(name, val),
        invalid: isInvalid,
        'aria-describedby': describedBy,
        'aria-invalid': isInvalid || undefined,
      });
    }

    if (React.isValidElement(children)) {
      const child = children as React.ReactElement<Record<string, unknown>>;
      return React.cloneElement(child, {
        id: child.props.id ?? fieldId,
        invalid: child.props.invalid ?? isInvalid,
        'aria-describedby': describedBy,
        'aria-invalid': isInvalid || undefined,
      });
    }

    return children;
  };

  return (
    <div
      className={cn(
        layout === 'horizontal'
          ? 'grid grid-cols-1 sm:grid-cols-3 gap-2 sm:gap-4 items-start'
          : 'space-y-1.5',
        className
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <label htmlFor={fieldId} className="text-xs sm:text-sm font-medium text-primary">
          {label}
          {required && (
            <span aria-hidden="true" className="ml-1 text-danger">
              *
            </span>
          )}
        </label>
        {optionalText && !required && (
          <span className="text-[11px] text-tertiary">({t('common.optional')})</span>
        )}
      </div>

      <div className={cn(layout === 'horizontal' && 'sm:col-span-2', 'space-y-1.5')}>
        {renderControl()}

        {hint && !resolvedError && (
          <p id={hintId} className="text-xs text-secondary">
            {hint}
          </p>
        )}

        {resolvedError && (
          <div
            id={errorId}
            role="alert"
            className="flex items-center gap-1.5 text-xs font-medium text-danger"
          >
            <AlertCircle className="h-3.5 w-3.5 shrink-0" />
            <span>{resolvedError}</span>
          </div>
        )}

        {resolvedWarning && (
          <div
            id={warningId}
            role="status"
            className="flex items-start gap-1.5 rounded-lg border border-warning/35 bg-warning/10 px-2.5 py-1.5 text-xs text-primary"
          >
            <AlertTriangle className="h-3.5 w-3.5 shrink-0 text-warning mt-0.5" />
            <span>{resolvedWarning}</span>
          </div>
        )}
      </div>
    </div>
  );
};
