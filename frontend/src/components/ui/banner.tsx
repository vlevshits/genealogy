import React, { useState } from 'react';
import { X, Info, AlertTriangle, AlertCircle } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';

export type BannerVariant = 'info' | 'warning' | 'danger' | 'brand';

export interface BannerProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: BannerVariant;
  dismissible?: boolean;
  onDismiss?: () => void;
  action?: React.ReactNode;
  ref?: React.Ref<HTMLDivElement>;
}

export const Banner: React.FC<BannerProps> = ({
  variant = 'info',
  dismissible = false,
  onDismiss,
  action,
  className,
  children,
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const [dismissed, setDismissed] = useState(false);

  if (dismissed) return null;

  const variantClasses: Record<BannerVariant, string> = {
    info: 'bg-surface-muted border-b border-border text-primary',
    brand: 'bg-brand text-surface',
    warning: 'bg-warning/15 border-b border-warning/30 text-primary',
    danger: 'bg-danger text-surface',
  };

  const icons: Record<BannerVariant, React.ReactNode> = {
    info: <Info className="h-4 w-4 text-brand shrink-0" />,
    brand: <Info className="h-4 w-4 text-surface shrink-0" />,
    warning: <AlertTriangle className="h-4 w-4 text-warning shrink-0" />,
    danger: <AlertCircle className="h-4 w-4 text-surface shrink-0" />,
  };

  return (
    <div
      ref={ref}
      role="region"
      aria-live="polite"
      className={cn(
        'w-full px-4 py-2.5 text-xs sm:text-sm flex items-center justify-between gap-4',
        variantClasses[variant],
        className
      )}
      {...props}
    >
      <div className="flex items-center gap-2.5 font-medium">
        {icons[variant]}
        <span>{children}</span>
      </div>
      <div className="flex items-center gap-2 shrink-0">
        {action}
        {dismissible && (
          <IconButton
            size="sm"
            label={t('common.close')}
            onClick={() => {
              setDismissed(true);
              onDismiss?.();
            }}
            className="h-7 w-7 min-h-7 min-w-7"
          >
            <X className="h-4 w-4" />
          </IconButton>
        )}
      </div>
    </div>
  );
};
