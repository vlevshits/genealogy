import React, { useCallback, useEffect, useId, useRef, useState } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';
import {
  getFocusableElements,
  handleDialogFocusTrapKeyDown,
  useModalOverlay,
} from './overlay-stack';

export { getFocusableElements, handleDialogFocusTrapKeyDown };

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
  'data-testid'?: string;
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
  'data-testid': dataTestId,
}) => {
  const { t } = useI18n();
  const titleId = useId();
  const descId = useId();
  const overlayRef = useRef<HTMLDivElement | null>(null);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const bodyRef = useRef<HTMLDivElement | null>(null);
  const [canScrollDown, setCanScrollDown] = useState(false);

  const updateScrollCue = useCallback(() => {
    const el = bodyRef.current;
    if (!el) {
      setCanScrollDown(false);
      return;
    }
    setCanScrollDown(el.scrollHeight - el.scrollTop - el.clientHeight > 6);
  }, []);

  useModalOverlay({
    open,
    modal: true,
    dismissible,
    overlayRef,
    panelRef: dialogRef,
    onOpenChange,
  });

  useEffect(() => {
    if (!open) {
      setCanScrollDown(false);
      return;
    }
    const raf =
      typeof requestAnimationFrame === 'function'
        ? requestAnimationFrame(updateScrollCue)
        : 0;
    updateScrollCue();
    const el = bodyRef.current;
    let observer: ResizeObserver | null = null;
    if (el && typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(updateScrollCue);
      observer.observe(el);
    }
    if (typeof window !== 'undefined') {
      window.addEventListener('resize', updateScrollCue);
    }
    return () => {
      if (raf && typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(raf);
      }
      observer?.disconnect();
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', updateScrollCue);
      }
    };
  }, [open, children, updateScrollCue]);

  if (!open) return null;

  const sizeClasses: Record<DialogSize, string> = {
    sm: 'max-w-[400px] w-full rounded-2xl',
    md: 'w-full h-full md:h-auto md:max-w-[560px] md:rounded-2xl',
    xl: 'w-full h-full md:h-auto md:max-w-[800px] md:rounded-2xl',
    full: 'w-[95vw] h-[90vh] rounded-2xl',
  };

  return (
    <div
      ref={overlayRef}
      className="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4"
    >
      <div
        aria-hidden="true"
        onClick={() => dismissible && onOpenChange(false)}
        className="fixed inset-0 bg-primary/40 backdrop-blur-xs transition-opacity"
      />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
        aria-labelledby={title ? titleId : undefined}
        aria-describedby={description ? descId : undefined}
        data-testid={dataTestId}
        className={cn(
          'relative z-10 flex flex-col border border-border bg-surface text-primary shadow-xl max-h-screen md:max-h-[85vh] focus:outline-none',
          sizeClasses[size],
          className
        )}
      >
        {(title || dismissible) && (
          <div className="flex items-start justify-between gap-4 border-b border-border px-6 py-4">
            <div className="min-w-0">
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
                className="h-10 w-10 min-h-10 min-w-10"
                label={t('common.close')}
                onClick={() => onOpenChange(false)}
              >
                <X className="h-4 w-4" />
              </IconButton>
            )}
          </div>
        )}
        <div className="relative flex flex-1 min-h-0 flex-col">
          <div
            ref={bodyRef}
            onScroll={updateScrollCue}
            className="flex-1 overflow-y-auto px-6 py-4 text-sm"
          >
            {children}
          </div>
          {canScrollDown && (
            <div
              aria-hidden="true"
              data-testid="dialog-scroll-fade"
              className="pointer-events-none absolute inset-x-0 bottom-0 h-8 bg-gradient-to-t from-surface via-surface/85 to-transparent"
            />
          )}
        </div>
        {footer && (
          <div className="flex flex-wrap items-center justify-end gap-2.5 border-t border-border bg-surface-muted/40 px-6 py-3.5 pb-[max(0.875rem,env(safe-area-inset-bottom,0px))] md:rounded-b-2xl">
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
