import React, { useEffect, useState } from 'react';
import {
  Fingerprint,
  KeyRound,
  Laptop,
  Lock,
  Plus,
  RefreshCw,
  Trash2,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  EmptyState,
  PasswordInput,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  deletePasskey,
  fetchPasskeys,
  fetchSessions,
  PasskeyCredentialDto,
  regenerateRecoveryCodes,
  renamePasskey,
  requestAddPasskeyChallenge,
  revokeOtherSessions,
  revokeSession,
  SessionDto,
  updatePassword,
  verifyAddPasskey,
} from '../../api/client';
import { createPasskeyCredential } from '../../lib/webauthn';
import { useAuth } from '../../lib/auth-context';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { RecoveryCodesCard } from '../auth/RecoveryCodesCard';

export interface SecurityPasskeysPanelProps {
  initialPasskeys?: PasskeyCredentialDto[];
  initialSessions?: SessionDto[];
  initialRemainingCodes?: number;
}

export const SecurityPasskeysPanel: React.FC<SecurityPasskeysPanelProps> = ({
  initialPasskeys,
  initialSessions,
  initialRemainingCodes,
}) => {
  const { t, locale } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const [passkeys, setPasskeys] = useState<PasskeyCredentialDto[]>(
    initialPasskeys ?? []
  );
  const [sessions, setSessions] = useState<SessionDto[]>(
    initialSessions ?? []
  );
  const [newDeviceName, setNewDeviceName] = useState('');
  const [editingPasskeyId, setEditingPasskeyId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [addingPasskey, setAddingPasskey] = useState(false);

  // Recovery codes state
  const [remainingCodes, setRemainingCodes] = useState<number>(
    initialRemainingCodes ?? user?.remaining_recovery_codes ?? 10
  );
  const [freshCodes, setFreshCodes] = useState<string[] | null>(null);
  const [regeneratingCodes, setRegeneratingCodes] = useState(false);

  // Password state
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [savingPassword, setSavingPassword] = useState(false);

  useEffect(() => {
    if (initialPasskeys !== undefined && initialSessions !== undefined) {
      return;
    }
    let cancelled = false;

    fetchPasskeys()
      .then((res) => {
        if (!cancelled) setPasskeys(res.items);
      })
      .catch(() => {});

    fetchSessions()
      .then((res) => {
        if (!cancelled) setSessions(res.items);
      })
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, [initialPasskeys, initialSessions]);

  const formatDate = (iso: string | null): string => {
    if (!iso) return t('settings.security.neverUsed');
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  const handleRegisterPasskey = async () => {
    if (addingPasskey) return;
    setAddingPasskey(true);
    try {
      const challenge = await requestAddPasskeyChallenge();
      const cred = await createPasskeyCredential(
        challenge,
        user?.email || 'user@family.local',
        user?.display_name || 'Family Member',
        newDeviceName.trim() || undefined
      );
      const saved = await verifyAddPasskey(cred);
      setPasskeys((prev) => [saved, ...prev]);
      setNewDeviceName('');
      addToast({
        type: 'success',
        title: t('settings.security.passkeyAddedToast'),
        description: saved.device_name,
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('settings.security.passkeyErrorTitle'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setAddingPasskey(false);
    }
  };

  const handleRenamePasskey = async (id: string) => {
    if (!editingName.trim()) return;
    try {
      const updated = await renamePasskey(id, {
        device_name: editingName.trim(),
      });
      setPasskeys((prev) =>
        prev.map((pk) => (pk.id === id ? updated : pk))
      );
      setEditingPasskeyId(null);
      addToast({
        type: 'success',
        title: t('settings.security.passkeyRenamedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('settings.security.passkeyErrorTitle'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleRemovePasskey = async (id: string) => {
    try {
      await deletePasskey(id);
      setPasskeys((prev) => prev.filter((pk) => pk.id !== id));
      addToast({
        type: 'success',
        title: t('settings.security.passkeyRemovedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('settings.security.passkeyErrorTitle'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleRegenerateCodes = async () => {
    if (regeneratingCodes) return;
    setRegeneratingCodes(true);
    try {
      const res = await regenerateRecoveryCodes();
      setFreshCodes(res.recovery_codes);
      setRemainingCodes(res.recovery_codes.length);
      addToast({
        type: 'success',
        title: t('settings.security.regeneratedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setRegeneratingCodes(false);
    }
  };

  const handleSavePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (savingPassword || newPassword.length < 10) return;
    setSavingPassword(true);
    try {
      await updatePassword({
        current_password: currentPassword || undefined,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      addToast({
        type: 'success',
        title: t('settings.security.passwordUpdatedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setSavingPassword(false);
    }
  };

  const handleRevokeSession = async (id: string) => {
    try {
      await revokeSession(id);
      setSessions((prev) => prev.filter((s) => s.id !== id));
      addToast({
        type: 'success',
        title: t('settings.security.sessionRevokedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleRevokeOtherSessions = async () => {
    try {
      const res = await revokeOtherSessions();
      setSessions((prev) => prev.filter((s) => s.is_current));
      addToast({
        type: 'success',
        title: t('settings.security.otherSessionsRevokedToast', {
          count: res.revoked_count,
        }),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  return (
    <div className="space-y-6">
      {/* 1. Registered Passkeys Card */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div className="space-y-1 min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <Fingerprint className="h-4.5 w-4.5 text-brand" />
              <h2 className="text-base font-semibold text-primary">
                {t('settings.security.passkeysTitle')}
              </h2>
            </div>
            <p className="text-xs text-secondary">
              {t('settings.security.passkeysSubtitle')}
            </p>
          </div>

          <div className="flex flex-wrap sm:flex-nowrap items-center justify-end gap-2 shrink-0 sm:ml-auto">
            <div className="w-48">
              <TextInput
                size="sm"
                value={newDeviceName}
                onChange={(e) => setNewDeviceName(e.target.value)}
                placeholder={t('settings.security.deviceNamePlaceholder')}
                aria-label={t('settings.security.deviceNameLabel')}
              />
            </div>
            <Button
              variant="primary"
              size="sm"
              loading={addingPasskey}
              onClick={handleRegisterPasskey}
              leftIcon={<Plus className="h-4 w-4" />}
              className="shrink-0"
            >
              {t('settings.security.registerPasskeyBtn')}
            </Button>
          </div>
        </CardHeader>

        <CardBody>
          {passkeys.length === 0 ? (
            <EmptyState
              icon={<Fingerprint className="h-6 w-6" />}
              title={t('settings.security.noPasskeysTitle')}
              description={t('settings.security.noPasskeysDesc')}
            />
          ) : (
            <div className="divide-y divide-border">
              {passkeys.map((pk) => (
                <div
                  key={pk.id}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 py-3.5 first:pt-0 last:pb-0"
                >
                  <div className="space-y-1 min-w-0">
                    {editingPasskeyId === pk.id ? (
                      <div className="flex items-center gap-2">
                        <TextInput
                          size="sm"
                          value={editingName}
                          onChange={(e) => setEditingName(e.target.value)}
                          aria-label={t('settings.security.deviceNameLabel')}
                        />
                        <Button
                          size="sm"
                          variant="primary"
                          onClick={() => handleRenamePasskey(pk.id)}
                        >
                          {t('common.save')}
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => setEditingPasskeyId(null)}
                        >
                          {t('common.cancel')}
                        </Button>
                      </div>
                    ) : (
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-semibold text-primary">
                          {pk.device_name}
                        </span>
                        <Badge variant="neutral" size="sm">
                          {pk.transports?.[0] || 'internal'}
                        </Badge>
                      </div>
                    )}
                    <div className="text-xs text-secondary">
                      {t('settings.security.enrolledLabel')}:{' '}
                      {formatDate(pk.created_at)} ·{' '}
                      {t('settings.security.lastUsedLabel')}:{' '}
                      {formatDate(pk.last_used_at)}
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    {editingPasskeyId !== pk.id && (
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => {
                          setEditingPasskeyId(pk.id);
                          setEditingName(pk.device_name);
                        }}
                      >
                        {t('settings.security.renameBtn')}
                      </Button>
                    )}
                    <Button
                      size="sm"
                      variant="danger"
                      leftIcon={<Trash2 className="h-3.5 w-3.5" />}
                      onClick={() => handleRemovePasskey(pk.id)}
                    >
                      {t('settings.security.removeBtn')}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>

      {/* 2. Emergency Recovery Codes Card */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div className="space-y-1 min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <KeyRound className="h-4.5 w-4.5 text-brand" />
              <h2 className="text-base font-semibold text-primary">
                {t('settings.security.recoveryTitle')}
              </h2>
            </div>
            <p className="text-xs text-secondary">
              {t('settings.security.recoverySubtitle')}
            </p>
          </div>
          <Badge
            variant={remainingCodes > 3 ? 'success' : 'warning'}
            className="shrink-0 sm:ml-auto"
          >
            {t('settings.security.recoveryStatus', {
              remaining: remainingCodes,
            })}
          </Badge>
        </CardHeader>

        <CardBody className="space-y-4">
          {freshCodes && (
            <RecoveryCodesCard
              codes={freshCodes}
              requireConfirmation={false}
            />
          )}
          <div>
            <Button
              variant="secondary"
              size="sm"
              loading={regeneratingCodes}
              leftIcon={<RefreshCw className="h-3.5 w-3.5" />}
              onClick={handleRegenerateCodes}
            >
              {t('settings.security.regenerateCodesBtn')}
            </Button>
          </div>
        </CardBody>
      </Card>

      {/* 3. Password Fallback Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Lock className="h-4.5 w-4.5 text-brand" />
            <h2 className="text-base font-semibold text-primary">
              {t('settings.security.passwordTitle')}
            </h2>
          </div>
          <p className="mt-1 text-xs text-secondary">
            {t('settings.security.passwordSubtitle')}
          </p>
        </CardHeader>
        <CardBody>
          <form onSubmit={handleSavePassword} className="space-y-4 max-w-md">
            <FormField
              name="current_password"
              label={t('settings.security.currentPasswordLabel')}
              optionalText
            >
              <PasswordInput
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
              />
            </FormField>

            <FormField
              name="new_password"
              label={t('settings.security.newPasswordLabel')}
              required
            >
              <PasswordInput
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                showStrengthHint
              />
            </FormField>

            <Button
              type="submit"
              variant="primary"
              size="sm"
              loading={savingPassword}
              disabled={savingPassword || newPassword.length < 10}
            >
              {t('settings.security.savePasswordBtn')}
            </Button>
          </form>
        </CardBody>
      </Card>

      {/* 4. Active Sessions Card */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div className="space-y-1 min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <Laptop className="h-4.5 w-4.5 text-brand" />
              <h2 className="text-base font-semibold text-primary">
                {t('settings.security.sessionsTitle')}
              </h2>
            </div>
            <p className="text-xs text-secondary">
              {t('settings.security.sessionsSubtitle')}
            </p>
          </div>
          {sessions.length > 1 && (
            <Button
              variant="secondary"
              size="sm"
              onClick={handleRevokeOtherSessions}
              className="shrink-0 sm:ml-auto"
            >
              {t('settings.security.signOutEverywhereBtn')}
            </Button>
          )}
        </CardHeader>
        <CardBody>
          <div className="divide-y divide-border">
            {sessions.map((sess) => (
              <div
                key={sess.id}
                className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 py-3 first:pt-0 last:pb-0"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-primary">
                      {sess.user_agent || 'Browser Session'}
                    </span>
                    {sess.is_current && (
                      <Badge variant="success" size="sm">
                        {t('settings.security.currentSessionBadge')}
                      </Badge>
                    )}
                  </div>
                  <div className="text-xs text-secondary">
                    {sess.ip_address ? `${sess.ip_address} · ` : ''}
                    {t('settings.security.lastSeenLabel')}:{' '}
                    {formatDate(sess.last_seen_at)}
                  </div>
                </div>
                {!sess.is_current && (
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => handleRevokeSession(sess.id)}
                  >
                    {t('settings.security.signOutSessionBtn')}
                  </Button>
                )}
              </div>
            ))}
          </div>
        </CardBody>
      </Card>
    </div>
  );
};
