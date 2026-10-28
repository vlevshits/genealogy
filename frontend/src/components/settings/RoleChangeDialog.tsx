import React, { useEffect, useState } from 'react';
import { ShieldAlert } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  RoleBadge,
  Select,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  AdminUserSummaryDto,
  ApiError,
  updateAdminUser,
} from '../../api/client';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';

export interface RoleChangeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  user: AdminUserSummaryDto | null;
  activeAdminCount: number;
  onUserUpdated?: (updated: AdminUserSummaryDto) => void;
}

export const RoleChangeDialog: React.FC<RoleChangeDialogProps> = ({
  open,
  onOpenChange,
  user,
  activeAdminCount,
  onUserUpdated,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const normalizeRole = (
    role?: string
  ): 'admin' | 'editor' | 'viewer' => {
    if (role === 'admin' || role === 'editor' || role === 'viewer') {
      return role;
    }
    return 'viewer';
  };

  const [selectedRole, setSelectedRole] = useState<'admin' | 'editor' | 'viewer'>(
    normalizeRole(user?.role)
  );
  const [saving, setSaving] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (user) {
      setSelectedRole(normalizeRole(user.role));
      setErrorMsg(null);
    }
  }, [user, open]);

  if (!user) return null;

  const isSoleActiveAdmin =
    user.role === 'admin' && user.is_active && activeAdminCount <= 1;

  const roleDescription =
    selectedRole === 'admin'
      ? t('settings.users.roleModal.roleAdminDesc')
      : selectedRole === 'editor'
      ? t('settings.users.roleModal.roleEditorDesc')
      : t('settings.users.roleModal.roleViewerDesc');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving || isSoleActiveAdmin) return;
    setSaving(true);
    setErrorMsg(null);
    try {
      const updated = await updateAdminUser(user.id, { role: selectedRole });
      onUserUpdated?.(updated);
      addToast({
        type: 'success',
        title: t('settings.users.roleUpdatedToast'),
        description: `${updated.display_name} · ${selectedRole}`,
      });
      onOpenChange(false);
    } catch (err) {
      if (err instanceof ApiError && err.code === 'sole_admin_protected') {
        setErrorMsg(t('rbac.soleAdminError'));
      } else {
        setErrorMsg(getLocalizedErrorMessage(err, t));
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={t('settings.users.roleModal.title')}
      description={t('settings.users.roleModal.subtitle', {
        name: user.display_name,
      })}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
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
            title={t('settings.users.roleModal.soleAdminWarningTitle')}
          >
            {t('settings.users.roleModal.soleAdminWarningDesc')}
          </Alert>
        )}

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <FormField
          name="member_role"
          label={t('settings.users.roleModal.roleSelectLabel')}
          hint={roleDescription}
        >
          <Select
            ariaLabel={t('settings.users.roleModal.roleSelectLabel')}
            value={selectedRole}
            disabled={isSoleActiveAdmin || saving}
            onChange={(val) =>
              setSelectedRole(val as 'admin' | 'editor' | 'viewer')
            }
            options={[
              { value: 'admin', label: t('settings.users.roleAdmin') },
              { value: 'editor', label: t('settings.users.roleEditor') },
              { value: 'viewer', label: t('settings.users.roleViewer') },
            ]}
          />
        </FormField>

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={saving}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={
              saving || isSoleActiveAdmin || selectedRole === user.role
            }
          >
            {t('settings.users.roleModal.saveBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
