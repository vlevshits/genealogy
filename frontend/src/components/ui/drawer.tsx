import React, { useId, useRef } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';
import { useModalOverlay } from './overlay-stack';

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
  'data-testid'?: string;
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
  'data-testid': dataTestId,
}) => {
  const { t } = useI18n();
  const titleId = useId();
  const descId = useId();
  const overlayRef = useRef<HTMLDivElement | null>(null);
  const drawerRef = useRef<HTMLElement | null>(null);

  useModalOverlay({
    open,
    modal,
    dismissible: true,
    overlayRef,
    panelRef: drawerRef,
    onOpenChange,
  });

  if (!open) return null;

  return (
    <div
      ref={overlayRef}
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
        ref={drawerRef}
        role="dialog"
        aria-modal={modal}
        aria-labelledby={title ? titleId : undefined}
        aria-describedby={description ? descId : undefined}
        data-testid={dataTestId}
        tabIndex={-1}
        className={cn(
          'relative z-10 flex flex-col border border-border bg-surface text-primary shadow-xl pointer-events-auto transition-transform duration-200 focus:outline-none',
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
          <div className="min-w-0">
            {title && (
              <h2 id={titleId} className="text-base font-semibold text-primary">
                {title}
              </h2>
            )}
            {description && (
              <p id={descId} className="mt-0.5 text-xs text-secondary">
                {description}
              </p>
            )}
          </div>
          <IconButton
            size="sm"
            className="h-10 w-10 min-h-10 min-w-10"
            label={t('common.close')}
            onClick={() => onOpenChange(false)}
          >
            <X className="h-4 w-4" />
          </IconButton>
        </div>
        <div className="flex-1 overflow-y-auto px-6 py-5 text-sm">{children}</div>
        {footer && (
          <div className="sticky bottom-0 flex flex-wrap items-center justify-end gap-2.5 border-t border-border bg-surface px-6 py-3.5 pb-[max(0.875rem,env(safe-area-inset-bottom,0px))]">
            {footer}
          </div>
        )}
      </aside>
    </div>
  );
};
