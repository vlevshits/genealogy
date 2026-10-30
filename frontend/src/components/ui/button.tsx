import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';
import { Spinner } from './spinner';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 font-medium rounded-lg transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none select-none cursor-pointer active:scale-[0.99]',
  {
    variants: {
      variant: {
        primary: 'bg-brand text-surface hover:opacity-95 shadow-xs',
        secondary:
          'bg-surface text-primary border border-border hover:bg-surface-muted shadow-xs',
        ghost: 'bg-transparent text-secondary hover:bg-surface-muted hover:text-primary',
        danger: 'bg-danger text-surface hover:opacity-95 shadow-xs',
        link: 'bg-transparent text-brand underline-offset-4 hover:underline p-0 h-auto',
      },
      size: {
        xs: 'min-h-6 h-6 px-2 py-0.5 text-[11px]',
        sm: 'min-h-8 px-3 py-1 text-xs touch-target-expand',
        md: 'min-h-11 px-4 py-2.5 text-sm',
        lg: 'min-h-12 px-6 py-2.5 text-base',
      },
      fullWidth: {
        true: 'w-full',
        false: '',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
      fullWidth: false,
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  loading?: boolean;
  asChild?: boolean;
  ref?: React.Ref<HTMLButtonElement>;
}

export const Button: React.FC<ButtonProps> = ({
  className,
  variant = 'primary',
  size = 'md',
  fullWidth = false,
  leftIcon,
  rightIcon,
  loading = false,
  disabled,
  asChild = false,
  children,
  type = 'button',
  ref,
  ...props
}) => {
  const isDisabled = Boolean(disabled || loading);
  const classes = cn(buttonVariants({ variant, size, fullWidth }), className);

  if (asChild && React.isValidElement(children)) {
    const child = children as React.ReactElement<Record<string, unknown>>;
    return React.cloneElement(child, {
      className: cn(classes, child.props.className as string | undefined),
      'aria-busy': loading || undefined,
      'aria-disabled': isDisabled || undefined,
    });
  }

  return (
    <button
      ref={ref}
      type={type}
      disabled={isDisabled}
      aria-busy={loading || undefined}
      className={classes}
      {...props}
    >
      {loading ? <Spinner size="sm" aria-hidden="true" /> : leftIcon}
      <span>{children}</span>
      {!loading && rightIcon}
    </button>
  );
};
