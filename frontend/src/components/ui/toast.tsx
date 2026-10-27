import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { CheckCircle2, AlertCircle, Info, RotateCcw, Undo2, X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';
import { Button } from './button';
import { IconButton } from './icon-button';

export type ToastType = 'success' | 'error' | 'info' | 'undo';

export interface ToastItem {
  id: string;
  type: ToastType;
  title: string;
  description?: string;
  onRetry?: () => void;
  onUndo?: () => void;
  durationSec?: number;
}

interface ToastContextValue {
  toasts: ToastItem[];
  addToast: (toast: Omit<ToastItem, 'id'>) => string;
  dismissToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const SingleToastCard: React.FC<{
  item: ToastItem;
  onDismiss: (id: string) => void;
}> = ({ item, onDismiss }) => {
  const { t } = useI18n();
  const totalSec = item.durationSec ?? (item.type === 'undo' ? 6 : 4);
  const [remaining, setRemaining] = useState(totalSec);

  useEffect(() => {
    if (remaining <= 0) {
      onDismiss(item.id);
    }
  }, [remaining, item.id, onDismiss]);

  useEffect(() => {
    const interval = setInterval(() => {
      setRemaining((prev) => Math.max(0, prev - 1));
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const progressPct = Math.max(0, Math.min(100, Math.round((remaining / totalSec) * 100)));

  return (
    <div
      role="status"
      aria-live="polite"
      data-testid={`toast-${item.type}`}
      className={cn(
        'relative overflow-hidden flex items-center justify-between gap-3 rounded-xl border border-border bg-surface px-4 py-3 text-xs sm:text-sm text-primary shadow-lg transition-all'
      )}
    >
      <div className="flex items-center gap-2.5 min-w-0">
        {item.type === 'success' && <CheckCircle2 className="h-4 w-4 text-success shrink-0" />}
        {item.type === 'error' && <AlertCircle className="h-4 w-4 text-danger shrink-0" />}
        {item.type === 'info' && <Info className="h-4 w-4 text-brand shrink-0" />}
        {item.type === 'undo' && (
          <span
            data-testid="undo-toast-countdown"
            className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-brand/15 font-mono text-[11px] font-bold text-brand shrink-0"
          >
            {remaining}s
          </span>
        )}
        <div className="min-w-0">
          <div className="font-medium truncate">{item.title}</div>
          {item.description && (
            <div className="text-xs text-secondary truncate">{item.description}</div>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        {item.type === 'undo' && item.onUndo && (
          <Button
            size="sm"
            variant="primary"
            leftIcon={<Undo2 className="h-3.5 w-3.5" />}
            onClick={() => {
              item.onUndo?.();
              onDismiss(item.id);
            }}
          >
            {t('common.undo')}
          </Button>
        )}
        {item.type === 'error' && item.onRetry && (
          <Button
            size="sm"
            variant="secondary"
            leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
            onClick={() => {
              item.onRetry?.();
              onDismiss(item.id);
            }}
          >
            {t('common.retry')}
          </Button>
        )}
        <IconButton
          size="sm"
          label={t('common.close')}
          onClick={() => onDismiss(item.id)}
        >
          <X className="h-3.5 w-3.5" />
        </IconButton>
      </div>

      {item.type === 'undo' && (
        <div
          aria-hidden="true"
          data-testid="undo-toast-progress"
          style={{ width: `${progressPct}%` }}
          className="absolute bottom-0 left-0 h-0.5 bg-brand transition-all duration-1000"
        />
      )}
    </div>
  );
};

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((item) => item.id !== id));
  }, []);

  const addToast = useCallback((toast: Omit<ToastItem, 'id'>) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    setToasts((prev) => [...prev.slice(-2), { ...toast, id }]);
    return id;
  }, []);

  return (
    <ToastContext.Provider value={{ toasts, addToast, dismissToast }}>
      {children}
      <div
        aria-live="polite"
        aria-label="Notifications"
        className="fixed bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] sm:bottom-4 left-1/2 z-50 flex w-full max-w-sm -translate-x-1/2 flex-col gap-2 px-4 sm:left-auto sm:right-4 sm:translate-x-0"
      >
        {toasts.map((item) => (
          <SingleToastCard key={item.id} item={item} onDismiss={dismissToast} />
        ))}
      </div>
    </ToastContext.Provider>
  );
};

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return ctx;
}
