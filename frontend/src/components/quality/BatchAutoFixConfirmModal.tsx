import React from 'react';
import { Sparkles } from 'lucide-react';
import { Button, Dialog } from '../ui';
import { useI18n } from '../../i18n';

export interface BatchAutoFixConfirmModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  highConfidenceCount: number;
  loading?: boolean;
  onConfirm: () => void;
}

export const BatchAutoFixConfirmModal: React.FC<
  BatchAutoFixConfirmModalProps
> = ({
  open,
  onOpenChange,
  highConfidenceCount,
  loading = false,
  onConfirm,
}) => {
  const { t } = useI18n();

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('quality.places.batchConfirmTitle')}
      description={t('quality.places.batchConfirmDesc', {
        count: highConfidenceCount,
      })}
      size="md"
      footer={
        <div className="flex items-center justify-end gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onOpenChange(false)}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            size="sm"
            loading={loading}
            data-testid="confirm-batch-autofix-btn"
            leftIcon={<Sparkles className="h-4 w-4" />}
            onClick={onConfirm}
          >
            {t('quality.places.batchConfirmBtn', {
              count: highConfidenceCount,
            })}
          </Button>
        </div>
      }
    >
      <div className="space-y-3 text-xs sm:text-sm text-secondary">
        <p>{t('quality.places.batchConfirmNote1')}</p>
        <p>{t('quality.places.batchConfirmNote2')}</p>
      </div>
    </Dialog>
  );
};
