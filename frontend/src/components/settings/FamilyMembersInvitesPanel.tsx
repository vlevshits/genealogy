import React, { useEffect, useState } from 'react';
import {
  Link2,
  Plus,
  Trash2,
  UserPlus,
  Users,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  CopyButton,
  Dialog,
  EmailInput,
  RoleBadge,
  Select,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  AdminSignInLinkResponse,
  AdminUserSummaryDto,
  createAdminInvite,
  createAdminSignInLink,
  fetchAdminInvites,
  fetchAdminUsers,
  InvitationDto,
  InviteCreatedResponse,
  revokeAdminInvite,
  revokeAdminUserSessions,
} from '../../api/client';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { RoleChangeDialog } from './RoleChangeDialog';
import { UserDeactivateDialog } from './UserDeactivateDialog';
import { UserTableRow } from './UserTableRow';

export interface FamilyMembersInvitesPanelProps {
  initialUsers?: AdminUserSummaryDto[];
  initialInvites?: InvitationDto[];
}

export const FamilyMembersInvitesPanel: React.FC<
  FamilyMembersInvitesPanelProps
> = ({ initialUsers, initialInvites }) => {
  const { t, locale } = useI18n();
  const { addToast } = useToast();

  const [users, setUsers] = useState<AdminUserSummaryDto[]>(
    initialUsers ?? []
  );
  const [invites, setInvites] = useState<InvitationDto[]>(
    initialInvites ?? []
  );

  // Invite Dialog state
  const [inviteModalOpen, setInviteModalOpen] = useState(false);
  const [inviteeName, setInviteeName] = useState('');
  const [inviteeEmail, setInviteeEmail] = useState('');
  const [inviteRole, setInviteRole] = useState<'editor' | 'viewer'>('editor');
  const [creatingInvite, setCreatingInvite] = useState(false);
  const [createdInvite, setCreatedInvite] =
    useState<InviteCreatedResponse | null>(null);

  // One-time 24h sign-in link state
  const [activeSignInLink, setActiveSignInLink] = useState<{
    userName: string;
    data: AdminSignInLinkResponse;
  } | null>(null);

  // Role Change & Deactivate Dialog state
  const [roleTargetUser, setRoleTargetUser] =
    useState<AdminUserSummaryDto | null>(null);
  const [deactivateTargetUser, setDeactivateTargetUser] =
    useState<AdminUserSummaryDto | null>(null);

  useEffect(() => {
    if (initialUsers !== undefined && initialInvites !== undefined) {
      return;
    }
    let cancelled = false;

    fetchAdminUsers()
      .then((res) => {
        if (!cancelled) setUsers(res.items);
      })
      .catch(() => {});

    fetchAdminInvites()
      .then((res) => {
        if (!cancelled) setInvites(res.items);
      })
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, [initialUsers, initialInvites]);

  const formatDate = (iso: string): string => {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  const handleCreateInvite = async (e: React.FormEvent) => {
    e.preventDefault();
    if (creatingInvite) return;
    setCreatingInvite(true);
    try {
      const res = await createAdminInvite({
        invitee_name: inviteeName.trim() || undefined,
        email: inviteeEmail.trim() || undefined,
        role: inviteRole,
      });
      setCreatedInvite(res);
      setInvites((prev) => [res.invitation, ...prev]);
      setInviteeName('');
      setInviteeEmail('');
      setInviteModalOpen(false);
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setCreatingInvite(false);
    }
  };

  const handleRevokeInvite = async (id: string) => {
    try {
      await revokeAdminInvite(id);
      setInvites((prev) => prev.filter((i) => i.id !== id));
      addToast({
        type: 'success',
        title: t('settings.users.inviteRevokedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleCreateSignInLink = async (u: AdminUserSummaryDto) => {
    try {
      const res = await createAdminSignInLink(u.id);
      setActiveSignInLink({
        userName: u.display_name,
        data: res,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleRevokeUserSessions = async (u: AdminUserSummaryDto) => {
    try {
      await revokeAdminUserSessions(u.id);
      addToast({
        type: 'success',
        title: t('settings.users.userSessionsRevokedToast'),
        description: u.display_name,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleUserUpdated = (updated: AdminUserSummaryDto) => {
    setUsers((prev) =>
      prev.map((item) => (item.id === updated.id ? updated : item))
    );
  };

  const activeAdminCount = users.filter(
    (u) => u.role === 'admin' && u.is_active
  ).length;
  const pendingInvites = invites.filter((i) => !i.used_at && !i.revoked_at);

  return (
    <div className="space-y-6">
      {/* Generated Invitation Link Banner */}
      {createdInvite && (
        <Alert
          variant="success"
          icon={<UserPlus className="h-4 w-4 text-success shrink-0 mt-0.5" />}
          title={t('settings.users.inviteCreatedTitle')}
          action={<CopyButton value={createdInvite.invite_url} />}
        >
          <div className="space-y-1">
            <div>{t('settings.users.inviteCreatedDesc')}</div>
            <code className="block font-mono text-xs text-primary break-all">
              {createdInvite.invite_url}
            </code>
          </div>
        </Alert>
      )}

      {/* Generated 24h Sign-In Link Banner */}
      {activeSignInLink && (
        <Alert
          variant="info"
          icon={<Link2 className="h-4 w-4 text-brand shrink-0 mt-0.5" />}
          title={`${t('settings.users.signInLinkTitle')} (${activeSignInLink.userName})`}
          action={<CopyButton value={activeSignInLink.data.sign_in_url} />}
        >
          <div className="space-y-1">
            <div>{t('settings.users.signInLinkDesc')}</div>
            <code className="block font-mono text-xs text-primary break-all">
              {activeSignInLink.data.sign_in_url}
            </code>
          </div>
        </Alert>
      )}

      {/* Family Members Table Card */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div className="space-y-1 min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <Users className="h-4.5 w-4.5 text-brand" />
              <h2 className="text-base font-semibold text-primary">
                {t('settings.users.title')}
              </h2>
            </div>
            <p className="text-xs text-secondary">
              {t('settings.users.subtitle')}
            </p>
          </div>

          <Button
            variant="primary"
            size="sm"
            leftIcon={<Plus className="h-4 w-4" />}
            onClick={() => setInviteModalOpen(true)}
            className="shrink-0 sm:ml-auto"
          >
            {t('settings.users.inviteBtn')}
          </Button>
        </CardHeader>

        <CardBody>
          <div className="divide-y divide-border">
            {users.map((u) => (
              <UserTableRow
                key={u.id}
                user={u}
                isSoleActiveAdmin={
                  u.role === 'admin' && u.is_active && activeAdminCount <= 1
                }
                onChangeRole={(target) => setRoleTargetUser(target)}
                onToggleActive={(target) => setDeactivateTargetUser(target)}
                onCreateSignInLink={handleCreateSignInLink}
                onRevokeSessions={handleRevokeUserSessions}
              />
            ))}
          </div>
        </CardBody>
      </Card>

      {/* Pending Invitations Card */}
      <Card>
        <CardHeader className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-primary">
            {t('settings.users.pendingInvitesTitle')}
          </h3>
          <Badge variant="neutral">{pendingInvites.length}</Badge>
        </CardHeader>

        <CardBody>
          {pendingInvites.length === 0 ? (
            <p className="text-xs text-secondary">
              {t('settings.users.noPendingInvites')}
            </p>
          ) : (
            <div className="divide-y divide-border">
              {pendingInvites.map((inv) => (
                <div
                  key={inv.id}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 py-3 first:pt-0 last:pb-0"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-primary">
                        {inv.invitee_name || inv.email || inv.id}
                      </span>
                      <RoleBadge role={inv.role} />
                    </div>
                    <div className="text-xs text-secondary">
                      {inv.email ? `${inv.email} · ` : ''}
                      {t('settings.users.expiresLabel')}:{' '}
                      {formatDate(inv.expires_at)}
                    </div>
                  </div>
                  <Button
                    size="sm"
                    variant="danger"
                    leftIcon={<Trash2 className="h-3.5 w-3.5" />}
                    onClick={() => handleRevokeInvite(inv.id)}
                  >
                    {t('settings.users.revokeInviteBtn')}
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>

      {/* Invite Family Member Modal */}
      <Dialog
        open={inviteModalOpen}
        onOpenChange={setInviteModalOpen}
        size="md"
        title={t('settings.users.inviteModalTitle')}
        description={t('settings.users.inviteModalDesc')}
      >
        <form onSubmit={handleCreateInvite} className="space-y-4">
          <FormField
            name="invitee_name"
            label={t('settings.users.inviteeNameLabel')}
            required
          >
            <TextInput
              value={inviteeName}
              onChange={(e) => setInviteeName(e.target.value)}
              placeholder={t('settings.users.inviteeNamePlaceholder')}
              required
            />
          </FormField>

          <FormField
            name="invitee_email"
            label={t('settings.users.inviteeEmailLabel')}
            optionalText
          >
            <EmailInput
              value={inviteeEmail}
              onChange={(e) => setInviteeEmail(e.target.value)}
            />
          </FormField>

          <FormField
            name="invite_role"
            label={t('settings.users.roleLabel')}
          >
            <Select
              ariaLabel={t('settings.users.roleLabel')}
              value={inviteRole}
              onChange={(val) => setInviteRole(val as 'editor' | 'viewer')}
              options={[
                { value: 'editor', label: t('settings.users.roleEditor') },
                { value: 'viewer', label: t('settings.users.roleViewer') },
              ]}
            />
          </FormField>

          <div className="flex justify-end gap-2 pt-2">
            <Button
              variant="secondary"
              onClick={() => setInviteModalOpen(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={creatingInvite}
              disabled={creatingInvite || !inviteeName.trim()}
            >
              {t('settings.users.generateInviteBtn')}
            </Button>
          </div>
        </form>
      </Dialog>

      {/* Role Change Modal */}
      <RoleChangeDialog
        open={Boolean(roleTargetUser)}
        onOpenChange={(open) => {
          if (!open) setRoleTargetUser(null);
        }}
        user={roleTargetUser}
        activeAdminCount={activeAdminCount}
        onUserUpdated={handleUserUpdated}
      />

      {/* Deactivate / Reactivate Modal */}
      <UserDeactivateDialog
        open={Boolean(deactivateTargetUser)}
        onOpenChange={(open) => {
          if (!open) setDeactivateTargetUser(null);
        }}
        user={deactivateTargetUser}
        activeAdminCount={activeAdminCount}
        onUserUpdated={handleUserUpdated}
      />
    </div>
  );
};
