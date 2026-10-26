import React from 'react';
import { cn } from '../../lib/utils';
import { Button } from '../ui/button';
import { useFormContext } from './form';
import { useI18n } from '../../i18n';

export interface FormActionsProps {
  onCancel?: () => void;
  saveLabel?: string;
  cancelLabel?: string;
  loading?: boolean;
  sticky?: boolean;
  className?: string;
}

export const FormActions: React.FC<FormActionsProps> = ({
  onCancel,
  saveLabel,
  cancelLabel,
  loading = false,
  sticky = false,
  className,
}) => {
  const { t } = useI18n();
  const form = useFormContext();
  const isDirty = form?.isDirty ?? false;

  return (
    <div
      className={cn(
        'flex items-center justify-between gap-4 border-t border-border pt-4',
        sticky &&
          'sticky bottom-0 z-20 rounded-xl border bg-surface/95 px-4 py-3 shadow-md backdrop-blur-md',
        className
      )}
    >
      <div className="text-xs text-secondary">
        {isDirty && (
          <span className="inline-flex items-center gap-1.5 font-medium text-warning">
            <span className="h-2 w-2 rounded-full bg-warning" />
            {t('common.unsavedChanges')}
          </span>
        )}
      </div>
      <div className="flex items-center gap-2.5">
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            form?.reset();
            onCancel?.();
          }}
        >
          {cancelLabel || t('common.cancel')}
        </Button>
        <Button type="submit" variant="primary" loading={loading}>
          {saveLabel || t('common.save')}
        </Button>
      </div>
    </div>
  );
};
