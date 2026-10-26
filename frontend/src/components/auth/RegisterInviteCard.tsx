import React, { useEffect, useState } from 'react';
import { Fingerprint, UserPlus } from 'lucide-react';
import {
  Alert,
  Button,
  EmailInput,
  PasswordInput,
  Spinner,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import {
  fetchInvitePreview,
  InvitePreviewResponse,
  registerWithInvite,
  requestRegisterChallenge,
} from '../../api/client';
import {
  createPasskeyCredential,
  isSecureContextAvailable,
} from '../../lib/webauthn';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import { RecoveryCodesCard } from './RecoveryCodesCard';

export interface RegisterInviteCardProps {
  inviteToken: string;
  initialInvite?: InvitePreviewResponse | null;
  onComplete?: () => void;
}

export const RegisterInviteCard: React.FC<RegisterInviteCardProps> = ({
  inviteToken,
  initialInvite,
  onComplete,
}) => {
  const { t } = useI18n();
  const { establishSession } = useAuth();

  const [invite, setInvite] = useState<InvitePreviewResponse | null>(
    initialInvite ?? null
  );
  const [loadingInvite, setLoadingInvite] = useState<boolean>(
    initialInvite === undefined
  );
  const [inviteError, setInviteError] = useState<string | null>(null);

  const initialNameParts = (initialInvite?.invitee_name ?? '').trim().split(/\s+/);
  const [firstName, setFirstName] = useState<string>(initialNameParts[0] ?? '');
  const [lastName, setLastName] = useState<string>(
    initialNameParts.slice(1).join(' ')
  );
  const [email, setEmail] = useState<string>(initialInvite?.email ?? '');
  const [password, setPassword] = useState<string>('');
  const [submitting, setSubmitting] = useState<boolean>(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [recoveryCodes, setRecoveryCodes] = useState<string[] | null>(null);

  const secureContext = isSecureContextAvailable();

  useEffect(() => {
    if (initialInvite !== undefined) return;
    if (!inviteToken) {
      setInviteError(t('auth.register.invalidInviteDesc'));
      setLoadingInvite(false);
      return;
    }

    let cancelled = false;
    fetchInvitePreview(inviteToken)
      .then((data) => {
        if (cancelled) return;
        setInvite(data);
        if (data.email) setEmail(data.email);
        if (data.invitee_name) {
          const parts = data.invitee_name.trim().split(/\s+/);
          setFirstName(parts[0] ?? '');
          setLastName(parts.slice(1).join(' '));
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setInviteError(
            err instanceof Error
              ? err.message
              : t('auth.register.invalidInviteDesc')
          );
        }
      })
      .finally(() => {
        if (!cancelled) setLoadingInvite(false);
      });

    return () => {
      cancelled = true;
    };
  }, [inviteToken, initialInvite, t]);

  if (loadingInvite) {
    return (
      <div className="flex items-center justify-center gap-2 py-8 text-sm text-secondary">
        <Spinner size="sm" />
        <span>{t('common.loading')}</span>
      </div>
    );
  }

  if (inviteError || !invite) {
    return (
      <Alert variant="danger" title={t('auth.register.invalidInviteTitle')}>
        {inviteError || t('auth.register.invalidInviteDesc')}
      </Alert>
    );
  }

  if (recoveryCodes && recoveryCodes.length > 0) {
    return (
      <RecoveryCodesCard
        codes={recoveryCodes}
        requireConfirmation
        onContinue={() => onComplete?.()}
      />
    );
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (submitting) return;
    setSubmitError(null);
    setSubmitting(true);

    try {
      let passkeyPayload = undefined;
      if (secureContext) {
        const challenge = await requestRegisterChallenge();
        passkeyPayload = await createPasskeyCredential(
          challenge,
          email.trim() || 'member@family.local',
          `${firstName.trim()} ${lastName.trim()}`.trim()
        );
      }

      const res = await registerWithInvite({
        invite_token: inviteToken,
        first_name: firstName.trim(),
        last_name: lastName.trim(),
        email: email.trim() || undefined,
        password: password.trim().length >= 10 ? password : undefined,
        passkey: passkeyPayload,
      });

      establishSession(res.user, res.session_token, false);
      setRecoveryCodes(res.recovery_codes);
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : t('common.errorLoading'));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Alert
        variant="info"
        icon={<UserPlus className="h-4 w-4 text-brand shrink-0 mt-0.5" />}
      >
        <div className="space-y-1">
          <div>
            {t('auth.register.invitedBanner', {
              inviter: invite.inviter_name,
              tree: invite.tree_title,
              role: invite.role,
            })}
          </div>
          {invite.person_id && (
            <div className="text-xs font-medium text-brand">
              {t('auth.register.linkedPersonBanner', {
                personId: invite.person_id,
              })}
            </div>
          )}
        </div>
      </Alert>

      {submitError && <Alert variant="danger">{submitError}</Alert>}

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <FormField
          name="first_name"
          label={t('auth.register.firstNameLabel')}
          required
        >
          <TextInput
            value={firstName}
            onChange={(e) => setFirstName(e.target.value)}
            required
          />
        </FormField>

        <FormField
          name="last_name"
          label={t('auth.register.lastNameLabel')}
          required
        >
          <TextInput
            value={lastName}
            onChange={(e) => setLastName(e.target.value)}
            required
          />
        </FormField>
      </div>

      <FormField
        name="email"
        label={t('auth.register.emailLabel')}
        required
      >
        <EmailInput
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
      </FormField>

      {invite.auth_password_enabled && (
        <FormField
          name="password"
          label={t('auth.register.passwordLabel')}
          optionalText={secureContext}
          required={!secureContext}
        >
          <PasswordInput
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            showStrengthHint
          />
        </FormField>
      )}

      <Button
        type="submit"
        variant="primary"
        size="lg"
        fullWidth
        loading={submitting}
        disabled={
          submitting ||
          !firstName.trim() ||
          !lastName.trim() ||
          !email.trim()
        }
        leftIcon={<Fingerprint className="h-4.5 w-4.5" />}
      >
        {t('auth.register.joinPasskeyCta')}
      </Button>
    </form>
  );
};
