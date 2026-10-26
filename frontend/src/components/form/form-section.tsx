import React from 'react';
import { cn } from '../../lib/utils';

export interface FormSectionProps extends React.HTMLAttributes<HTMLFieldSetElement> {
  title: string;
  description?: string;
}

export const FormSection: React.FC<FormSectionProps> = ({
  title,
  description,
  className,
  children,
  ...props
}) => {
  return (
    <fieldset
      className={cn('space-y-4 rounded-2xl border border-border bg-surface p-5', className)}
      {...props}
    >
      <div className="border-b border-border pb-3">
        <legend className="text-sm font-semibold text-primary">{title}</legend>
        {description && <p className="mt-0.5 text-xs text-secondary">{description}</p>}
      </div>
      <div className="space-y-4">{children}</div>
    </fieldset>
  );
};
