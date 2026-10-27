import { useCallback } from 'react';
import { undoTransaction } from '../api/client';
import { useToast } from '../components/ui';
import { useI18n } from '../i18n';

export interface TriggerUndoToastOptions {
  txId?: string | null;
  title?: string;
  description?: string;
  durationSec?: number;
  onUndone?: () => void;
}

export function useUndoToast() {
  const { addToast } = useToast();
  const { t } = useI18n();

  const showUndoToast = useCallback(
    ({
      txId,
      title,
      description,
      durationSec = 6,
      onUndone,
    }: TriggerUndoToastOptions = {}) => {
      return addToast({
        type: 'undo',
        durationSec,
        title: title || t('history.undo_toast_label'),
        description,
        onUndo: () => {
          const runUndo = async () => {
            try {
              if (txId) {
                await undoTransaction(txId);
              }
              onUndone?.();
              addToast({
                type: 'success',
                title: t('history.undo_confirmed_toast'),
              });
            } catch (err) {
              addToast({
                type: 'error',
                title: t('common.errorLoading'),
                description: err instanceof Error ? err.message : undefined,
              });
            }
          };
          void runUndo();
        },
      });
    },
    [addToast, t]
  );

  return { showUndoToast };
}
