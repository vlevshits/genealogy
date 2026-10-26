import React from 'react';
import {
  Link2,
  LogOut,
  Shield,
  UserCheck,
  UserX,
} from 'lucide-react';
import {
  Avatar,
  Badge,
  Button,
  RoleBadge,
} from '../ui';
import { AdminUserSummaryDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface UserTableRowProps {
  user: AdminUserSummaryDto;
  isSoleActiveAdmin?: boolean;
  onChangeRole: (user: AdminUserSummaryDto) => void;
  onToggleActive: (user: AdminUserSummaryDto) => void;
  onCreateSignInLink: (user: AdminUserSummaryDto) => void;
  onRevokeSessions: (user: AdminUserSummaryDto) => void;
}

export const UserTableRow: React.FC<UserTableRowProps> = ({
  user,
  isSoleActiveAdmin = false,
  onChangeRole,
  onToggleActive,
  onCreateSignInLink,
  onRevokeSessions,
}) => {
  const { t } = useI18n();

  return (
    <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3 py-3.5 first:pt-0 last:pb-0">
      <div className="flex items-center gap-3 min-w-0">
        <Avatar name={user.display_name} size="md" />
        <div className="min-w-0 space-y-0.5">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-semibold text-primary truncate">
              {user.display_name}
            </span>
            <RoleBadge role={user.role} isActive={user.is_active} />
            {user.person_id && (
              <Badge variant="neutral" size="sm">
                {t('settings.users.linkedPersonBadge', {
                  personId: user.person_id,
                })}
              </Badge>
            )}
            <Badge variant="neutral" size="sm">
              {t('settings.users.devicesCount', {
                count: user.passkeys_count,
              })}
            </Badge>
          </div>
          <div className="font-mono text-xs text-secondary truncate">
            {user.email}
          </div>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2 shrink-0">
        <Button
          size="sm"
          variant="secondary"
          leftIcon={<Shield className="h-3.5 w-3.5" />}
          onClick={() => onChangeRole(user)}
        >
          {t('settings.users.changeRoleBtn')}
        </Button>

        {user.is_active && (
          <Button
            size="sm"
            variant="secondary"
            leftIcon={<Link2 className="h-3.5 w-3.5" />}
            onClick={() => onCreateSignInLink(user)}
          >
            {t('settings.users.createSignInLinkBtn')}
          </Button>
        )}

        <Button
          size="sm"
          variant="ghost"
          leftIcon={<LogOut className="h-3.5 w-3.5" />}
          onClick={() => onRevokeSessions(user)}
        >
          {t('settings.users.revokeSessionsBtn')}
        </Button>

        <Button
          size="sm"
          variant={user.is_active ? 'ghost' : 'secondary'}
          disabled={user.is_active && isSoleActiveAdmin}
          leftIcon={
            user.is_active ? (
              <UserX className="h-3.5 w-3.5 text-danger" />
            ) : (
              <UserCheck className="h-3.5 w-3.5 text-success" />
            )
          }
          onClick={() => onToggleActive(user)}
        >
          {user.is_active
            ? t('settings.users.deactivateBtn')
            : t('settings.users.reactivateBtn')}
        </Button>
      </div>
    </div>
  );
};
