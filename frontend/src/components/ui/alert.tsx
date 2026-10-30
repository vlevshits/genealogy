import React from 'react';
import { AlertCircle, CheckCircle2, Info, AlertTriangle } from 'lucide-react';
import { cn } from '../../lib/utils';

export type AlertVariant = 'info' | 'success' | 'warning' | 'danger';

export interface AlertProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: AlertVariant;
  title?: string;
  action?: React.ReactNode;
  icon?: React.ReactNode;
  ref?: React.Ref<HTMLDivElement>;
}

export const Alert: React.FC<AlertProps> = ({
  variant = 'info',
  title,
  action,
  icon,
  className,
  children,
  ref,
  ...props
}) => {
  const styles: Record<AlertVariant, string> = {
    info: 'border-brand/30 bg-brand/10 text-primary',
    success: 'border-success/30 bg-success/10 text-primary',
    warning: 'border-warning/35 bg-warning/10 text-primary',
    danger: 'border-danger/35 bg-danger/10 text-primary',
  };

  const defaultIcons: Record<AlertVariant, React.ReactNode> = {
    info: <Info className="h-4 w-4 text-brand shrink-0 mt-0.5" />,
    success: <CheckCircle2 className="h-4 w-4 text-success shrink-0 mt-0.5" />,
    warning: <AlertTriangle className="h-4 w-4 text-warning shrink-0 mt-0.5" />,
    danger: <AlertCircle className="h-4 w-4 text-danger shrink-0 mt-0.5" />,
  };

  return (
    <div
      ref={ref}
      role={variant === 'danger' || variant === 'warning' ? 'alert' : 'status'}
      className={cn(
        'flex items-start justify-between gap-3 rounded-xl border p-4 text-xs sm:text-sm',
        styles[variant],
        className
      )}
      {...props}
    >
      <div className="flex items-start gap-3 min-w-0 flex-1">
        {icon ?? defaultIcons[variant]}
        <div className="space-y-1 min-w-0 flex-1">
          {title && <div className="font-semibold text-primary">{title}</div>}
          {children && <div className="text-secondary leading-relaxed">{children}</div>}
        </div>
      </div>
      {action && <div className="shrink-0">{action}</div>}
    </div>
  );
};
