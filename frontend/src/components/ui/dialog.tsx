import React, { useEffect, useId, useRef } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';

export type DialogSize = 'sm' | 'md' | 'xl' | 'full';

export interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  size?: DialogSize;
  dismissible?: boolean;
  title?: string;
  description?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  className?: string;
}

export const Dialog: React.FC<DialogProps> = ({
  open,
  onOpenChange,
  size = 'md',
  dismissible = true,
  title,
  description,
  children,
  footer,
  className,
}) => {
  const { t } = useI18n();
  const titleId = useId();
  const descId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const previousActiveRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) return;
    previousActiveRef.current = document.activeElement as HTMLElement | null;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && dismissible) {
        e.preventDefault();
        onOpenChange(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      previousActiveRef.current?.focus?.();
    };
  }, [open, dismissible, onOpenChange]);

  if (!open) return null;

  const sizeClasses: Record<DialogSize, string> = {
    sm: 'max-w-[400px] w-full rounded-2xl',
    md: 'w-full h-full md:h-auto md:max-w-[560px] md:rounded-2xl',
    xl: 'w-full h-full md:h-auto md:max-w-[800px] md:rounded-2xl',
    full: 'w-[95vw] h-[90vh] rounded-2xl',
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4">
      <div
        aria-hidden="true"
        onClick={() => dismissible && onOpenChange(false)}
        className="fixed inset-0 bg-primary/40 backdrop-blur-xs transition-opacity"
      />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? titleId : undefined}
        aria-describedby={description ? descId : undefined}
        className={cn(
          'relative z-10 flex flex-col border border-border bg-surface text-primary shadow-xl max-h-screen md:max-h-[85vh]',
          sizeClasses[size],
          className
        )}
      >
        {(title || dismissible) && (
          <div className="flex items-start justify-between gap-4 border-b border-border px-6 py-4">
            <div>
              {title && (
                <h2 id={titleId} className="text-base font-semibold text-primary">
                  {title}
                </h2>
              )}
              {description && (
                <p id={descId} className="mt-1 text-xs text-secondary">
                  {description}
                </p>
              )}
            </div>
            {dismissible && (
              <IconButton
                size="sm"
                label={t('common.close')}
                onClick={() => onOpenChange(false)}
              >
                <X className="h-4 w-4" />
              </IconButton>
            )}
          </div>
        )}
        <div className="flex-1 overflow-y-auto px-6 py-5 text-sm">{children}</div>
        {footer && (
          <div className="flex items-center justify-end gap-2.5 border-t border-border bg-surface-muted/40 px-6 py-3.5 rounded-b-2xl">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
};

export const DialogHeader: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('space-y-1', className)} {...props}>
    {children}
  </div>
);

export const DialogBody: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('space-y-4', className)} {...props}>
    {children}
  </div>
);

export const DialogFooter: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('flex items-center justify-end gap-2', className)} {...props}>
    {children}
  </div>
);
