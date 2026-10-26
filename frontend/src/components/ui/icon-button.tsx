import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';
import { Spinner } from './spinner';

const iconButtonVariants = cva(
  'inline-flex items-center justify-center rounded-lg transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none cursor-pointer shrink-0',
  {
    variants: {
      variant: {
        ghost: 'bg-transparent text-secondary hover:bg-surface-muted hover:text-primary',
        secondary: 'bg-surface text-primary border border-border hover:bg-surface-muted shadow-xs',
        floating:
          'bg-surface/90 backdrop-blur-md text-primary border border-border rounded-full hover:bg-surface shadow-sm',
        danger: 'bg-danger/10 text-danger hover:bg-danger hover:text-surface',
      },
      size: {
        sm: 'h-8 w-8 min-h-8 min-w-8 text-xs touch-target-expand',
        md: 'h-11 w-11 min-h-11 min-w-11 text-sm',
        lg: 'h-12 w-12 min-h-12 min-w-12 text-base',
      },
    },
    defaultVariants: {
      variant: 'ghost',
      size: 'md',
    },
  }
);

export interface IconButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'aria-label'>,
    VariantProps<typeof iconButtonVariants> {
  /** Required accessible label used for both aria-label and native title tooltip */
  label: string;
  loading?: boolean;
  ref?: React.Ref<HTMLButtonElement>;
}

export const IconButton: React.FC<IconButtonProps> = ({
  label,
  variant = 'ghost',
  size = 'md',
  loading = false,
  disabled,
  className,
  children,
  type = 'button',
  ref,
  ...props
}) => {
  return (
    <button
      ref={ref}
      type={type}
      aria-label={label}
      title={label}
      disabled={Boolean(disabled || loading)}
      aria-busy={loading || undefined}
      className={cn(iconButtonVariants({ variant, size }), className)}
      {...props}
    >
      {loading ? <Spinner size="sm" /> : children}
    </button>
  );
};
