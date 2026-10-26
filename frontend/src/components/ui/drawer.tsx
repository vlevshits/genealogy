import React, { useEffect, useId } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';

export interface DrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  side?: 'right' | 'bottom';
  modal?: boolean;
  title?: string;
  description?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  className?: string;
}

export const Drawer: React.FC<DrawerProps> = ({
  open,
  onOpenChange,
  side = 'right',
  modal = true,
  title,
  description,
  children,
  footer,
  className,
}) => {
  const { t } = useI18n();
  const titleId = useId();

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onOpenChange(false);
      }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [open, onOpenChange]);

  if (!open) return null;

  return (
    <div
      className={cn(
        'fixed inset-0 z-50 flex',
        !modal && 'pointer-events-none',
        side === 'right' ? 'justify-end' : 'items-end justify-center'
      )}
    >
      {modal && (
        <div
          aria-hidden="true"
          onClick={() => onOpenChange(false)}
          className="fixed inset-0 bg-primary/30 backdrop-blur-xs transition-opacity"
        />
      )}
      <aside
        role="dialog"
        aria-modal={modal}
        aria-labelledby={title ? titleId : undefined}
        className={cn(
          'relative z-10 flex flex-col border border-border bg-surface text-primary shadow-xl pointer-events-auto transition-transform duration-200',
          side === 'right'
            ? 'h-full w-full sm:w-[480px] border-l'
            : 'w-full max-h-[85vh] rounded-t-2xl border-t pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]',
          className
        )}
      >
        {side === 'bottom' && (
          <div className="flex justify-center pt-2.5 pb-1" aria-hidden="true">
            <div className="h-1.5 w-10 rounded-full bg-border" />
          </div>
        )}
        <div className="flex items-start justify-between gap-4 border-b border-border px-6 py-4">
          <div>
            {title && (
              <h2 id={titleId} className="text-base font-semibold text-primary">
                {title}
              </h2>
            )}
            {description && <p className="mt-0.5 text-xs text-secondary">{description}</p>}
          </div>
          <IconButton
            size="sm"
            label={t('common.close')}
            onClick={() => onOpenChange(false)}
          >
            <X className="h-4 w-4" />
          </IconButton>
        </div>
        <div className="flex-1 overflow-y-auto px-6 py-5 text-sm">{children}</div>
        {footer && (
          <div className="sticky bottom-0 flex items-center justify-end gap-2.5 border-t border-border bg-surface px-6 py-3.5">
            {footer}
          </div>
        )}
      </aside>
    </div>
  );
};
