import React, { useState } from 'react';
import { ShieldAlert, UserCheck, UserX } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  RoleBadge,
  useToast,
} from '../ui';
import {
  AdminUserSummaryDto,
  ApiError,
  updateAdminUser,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface UserDeactivateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  user: AdminUserSummaryDto | null;
  activeAdminCount: number;
  onUserUpdated?: (updated: AdminUserSummaryDto) => void;
}

export const UserDeactivateDialog: React.FC<UserDeactivateDialogProps> = ({
  open,
  onOpenChange,
  user,
  activeAdminCount,
  onUserUpdated,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();
  const [submitting, setSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  if (!user) return null;

  const isDeactivating = user.is_active;
  const isSoleActiveAdmin =
    isDeactivating && user.role === 'admin' && activeAdminCount <= 1;

  const handleConfirm = async () => {
    if (submitting || isSoleActiveAdmin) return;
    setSubmitting(true);
    setErrorMsg(null);
    try {
      const updated = await updateAdminUser(user.id, {
        is_active: !user.is_active,
      });
      onUserUpdated?.(updated);
      addToast({
        type: 'success',
        title: isDeactivating
          ? t('settings.users.userDeactivatedToast')
          : t('settings.users.userReactivatedToast'),
        description: updated.display_name,
      });
      onOpenChange(false);
    } catch (err) {
      if (err instanceof ApiError && err.code === 'sole_admin_protected') {
        setErrorMsg(t('rbac.soleAdminError'));
      } else {
        setErrorMsg(
          err instanceof Error ? err.message : t('common.errorLoading')
        );
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={
        isDeactivating
          ? t('settings.users.deactivateModal.deactivateTitle')
          : t('settings.users.deactivateModal.reactivateTitle')
      }
      description={
        isDeactivating
          ? t('settings.users.deactivateModal.deactivateDesc', {
              name: user.display_name,
            })
          : t('settings.users.deactivateModal.reactivateDesc', {
              name: user.display_name,
            })
      }
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between rounded-xl border border-border bg-surface-muted/50 px-3.5 py-2.5 text-xs">
          <div className="min-w-0">
            <div className="font-semibold text-primary truncate">
              {user.display_name}
            </div>
            <div className="font-mono text-secondary truncate">{user.email}</div>
          </div>
          <RoleBadge role={user.role} isActive={user.is_active} />
        </div>

        {isSoleActiveAdmin && (
          <Alert
            variant="warning"
            icon={<ShieldAlert className="h-4 w-4 text-warning shrink-0 mt-0.5" />}
          >
            {t('settings.users.deactivateModal.soleAdminWarning')}
          </Alert>
        )}

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={submitting}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant={isDeactivating ? 'danger' : 'primary'}
            loading={submitting}
            disabled={submitting || isSoleActiveAdmin}
            leftIcon={
              isDeactivating ? (
                <UserX className="h-4 w-4" />
              ) : (
                <UserCheck className="h-4 w-4" />
              )
            }
            onClick={handleConfirm}
          >
            {isDeactivating
              ? t('settings.users.deactivateModal.confirmDeactivateBtn')
              : t('settings.users.deactivateModal.confirmReactivateBtn')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
};
