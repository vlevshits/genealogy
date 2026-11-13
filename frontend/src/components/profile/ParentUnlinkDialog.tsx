import React from 'react';
import { Button, Dialog } from '../ui';
import { useI18n } from '../../i18n';

export interface ParentUnlinkDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Display name of the parent whose card was clicked. */
  parentName: string;
  /** Display name of the other parent. */
  otherParentName: string;
  onUnlinkOnly: () => void;
  onUnlinkBoth: () => void;
}

/** Two-option unlink choice shown when the parent family has two parents. */
export const ParentUnlinkDialog: React.FC<ParentUnlinkDialogProps> = ({
  open,
  onOpenChange,
  parentName,
  otherParentName,
  onUnlinkOnly,
  onUnlinkBoth,
}) => {
  const { t } = useI18n();
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={t('profile.family.unlinkParentChoiceTitle', { name: parentName })}
      description={t('profile.family.unlinkParentChoiceDesc')}
    >
      <div className="space-y-3">
        <div className="space-y-1 rounded-2xl border border-border p-3">
          <p className="text-sm text-secondary">
            {t('profile.family.unlinkOnlyDesc', { other: otherParentName })}
          </p>
          <Button variant="secondary" fullWidth onClick={onUnlinkOnly}>
            {t('profile.family.unlinkOnlyBtn', { name: parentName })}
          </Button>
        </div>
        <div className="space-y-1 rounded-2xl border border-border p-3">
          <p className="text-sm text-secondary">
            {t('profile.family.unlinkBothDesc')}
          </p>
          <Button variant="danger" fullWidth onClick={onUnlinkBoth}>
            {t('profile.family.unlinkBothBtn')}
          </Button>
        </div>
        <div className="flex justify-end pt-1">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
};
