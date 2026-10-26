import React, { useState } from 'react';
import {
  Fingerprint,
  KeyRound,
  Lock,
  Mail,
  Send,
} from 'lucide-react';
import {
  Alert,
  Button,
  Divider,
  EmailInput,
  PasswordInput,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import {
  loginWithPassword,
  loginWithRecoveryCode,
  requestLoginChallenge,
  requestMagicLink,
  verifyPasskeyLogin,
} from '../../api/client';
import {
  getPasskeyAssertion,
  isSecureContextAvailable,
} from '../../lib/webauthn';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';

export type LoginViewMode = 'default' | 'magic_sent' | 'recovery';

export interface LoginCardProps {
  authPasswordEnabled?: boolean;
  initialMode?: LoginViewMode;
  onSuccess?: (promptPasskeyEnrollment?: boolean) => void;
  onNavigate?: (path: string) => void;
}

export const LoginCard: React.FC<LoginCardProps> = ({
  authPasswordEnabled = true,
  initialMode = 'default',
  onSuccess,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { establishSession } = useAuth();

  const [mode, setMode] = useState<LoginViewMode>(initialMode);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [recoveryCode, setRecoveryCode] = useState('');
  const [magicPreviewUrl, setMagicPreviewUrl] = useState<string | null>(null);
  const [loadingPasskey, setLoadingPasskey] = useState(false);
  const [loadingPassword, setLoadingPassword] = useState(false);
  const [loadingMagic, setLoadingMagic] = useState(false);
  const [loadingRecovery, setLoadingRecovery] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const secureContext = isSecureContextAvailable();

  const handlePasskeySignIn = async () => {
    if (loadingPasskey) return;
    setError(null);
    setLoadingPasskey(true);
    try {
      const challenge = await requestLoginChallenge({
        email: email.trim() || undefined,
      });
      const assertion = await getPasskeyAssertion(challenge);
      const res = await verifyPasskeyLogin(assertion);
      establishSession(res.user, res.session_token, res.prompt_passkey_enrollment);
      onSuccess?.(res.prompt_passkey_enrollment);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('common.errorLoading'));
    } finally {
      setLoadingPasskey(false);
    }
  };

  const handlePasswordSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    if (loadingPassword || !email.trim() || !password) return;
    setError(null);
    setLoadingPassword(true);
    try {
      const res = await loginWithPassword({
        email: email.trim(),
        password,
      });
      establishSession(res.user, res.session_token, res.prompt_passkey_enrollment);
      onSuccess?.(res.prompt_passkey_enrollment);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('common.errorLoading'));
    } finally {
      setLoadingPassword(false);
    }
  };

  const handleRequestMagicLink = async () => {
    if (loadingMagic || !email.trim()) return;
    setError(null);
    setLoadingMagic(true);
    try {
      const res = await requestMagicLink({ email: email.trim() });
      setMagicPreviewUrl(res.preview_url ?? null);
      setMode('magic_sent');
    } catch (err) {
      setError(err instanceof Error ? err.message : t('common.errorLoading'));
    } finally {
      setLoadingMagic(false);
    }
  };

  const handleRecoverySubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (loadingRecovery || !email.trim() || !recoveryCode.trim()) return;
    setError(null);
    setLoadingRecovery(true);
    try {
      const res = await loginWithRecoveryCode({
        email: email.trim(),
        code: recoveryCode.trim(),
      });
      establishSession(res.user, res.session_token, res.prompt_passkey_enrollment);
      onSuccess?.(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('common.errorLoading'));
    } finally {
      setLoadingRecovery(false);
    }
  };

  if (mode === 'magic_sent') {
    return (
      <div className="space-y-4">
        <Alert
          variant="success"
          icon={<Mail className="h-4 w-4 text-success shrink-0 mt-0.5" />}
          title={t('auth.login.magicLinkSentTitle')}
        >
          {t('auth.login.magicLinkSentDesc', {
            email: email.trim() || 'user@example.com',
          })}
        </Alert>

        {magicPreviewUrl && (
          <Button
            variant="primary"
            fullWidth
            leftIcon={<Send className="h-4 w-4" />}
            onClick={() => onNavigate?.(magicPreviewUrl)}
          >
            {t('auth.login.magicLinkPreviewCta')}
          </Button>
        )}

        <Button
          variant="secondary"
          fullWidth
          onClick={() => setMode('default')}
        >
          {t('auth.login.useAnotherMethodBtn')}
        </Button>
      </div>
    );
  }

  if (mode === 'recovery') {
    return (
      <form onSubmit={handleRecoverySubmit} className="space-y-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-sm font-semibold text-primary">
            <KeyRound className="h-4 w-4 text-brand" />
            <span>{t('auth.login.recoveryTitle')}</span>
          </div>
          <p className="text-xs text-secondary">
            {t('auth.login.recoveryDesc')}
          </p>
        </div>

        {error && <Alert variant="danger">{error}</Alert>}

        <FormField
          name="recovery_email"
          label={t('auth.login.emailLabel')}
          required
        >
          <EmailInput
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder={t('auth.login.emailPlaceholder')}
            required
          />
        </FormField>

        <FormField
          name="recovery_code"
          label={t('auth.login.recoveryCodeLabel')}
          required
        >
          <TextInput
            value={recoveryCode}
            onChange={(e) => setRecoveryCode(e.target.value.toUpperCase())}
            placeholder={t('auth.login.recoveryCodePlaceholder')}
            className="font-mono tracking-wider"
            required
          />
        </FormField>

        <Button
          type="submit"
          variant="primary"
          fullWidth
          loading={loadingRecovery}
          disabled={loadingRecovery || !email.trim() || !recoveryCode.trim()}
          leftIcon={<KeyRound className="h-4 w-4" />}
        >
          {t('auth.login.recoverySubmitBtn')}
        </Button>

        <Button
          variant="ghost"
          size="sm"
          fullWidth
          onClick={() => {
            setError(null);
            setMode('default');
          }}
        >
          {t('auth.login.backToLoginBtn')}
        </Button>
      </form>
    );
  }

  return (
    <div className="space-y-5">
      {!secureContext ? (
        <Alert variant="warning" title={t('auth.insecureContextTitle')}>
          {t('auth.insecureContextDesc')}
        </Alert>
      ) : (
        <div className="space-y-1.5">
          <Button
            variant="primary"
            size="lg"
            fullWidth
            loading={loadingPasskey}
            onClick={handlePasskeySignIn}
            leftIcon={<Fingerprint className="h-5 w-5" />}
          >
            {t('auth.login.passkeyCta')}
          </Button>
          <p className="text-center text-[11px] text-tertiary">
            {t('auth.login.passkeyHint')}
          </p>
        </div>
      )}

      {error && <Alert variant="danger">{error}</Alert>}

      <Divider label={t('auth.login.orEmailDivider')} />

      <form onSubmit={handlePasswordSignIn} className="space-y-3">
        <FormField
          name="email"
          label={t('auth.login.emailLabel')}
          required
        >
          <EmailInput
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder={t('auth.login.emailPlaceholder')}
            autoComplete="username webauthn"
            required
          />
        </FormField>

        {authPasswordEnabled && (
          <FormField
            name="password"
            label={t('auth.login.passwordLabel')}
          >
            <PasswordInput
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
            />
          </FormField>
        )}

        <div className="flex flex-col gap-2 pt-1">
          {authPasswordEnabled && (
            <Button
              type="submit"
              variant="secondary"
              fullWidth
              loading={loadingPassword}
              disabled={loadingPassword || !email.trim() || !password}
              leftIcon={<Lock className="h-4 w-4" />}
            >
              {t('auth.login.passwordSignInBtn')}
            </Button>
          )}

          <Button
            type="button"
            variant="secondary"
            fullWidth
            loading={loadingMagic}
            disabled={loadingMagic || !email.trim()}
            leftIcon={<Mail className="h-4 w-4" />}
            onClick={handleRequestMagicLink}
          >
            {t('auth.login.magicLinkBtn')}
          </Button>
        </div>
      </form>

      <div className="pt-2 text-center">
        <Button
          variant="link"
          size="sm"
          leftIcon={<KeyRound className="h-3.5 w-3.5" />}
          onClick={() => {
            setError(null);
            setMode('recovery');
          }}
        >
          {t('auth.login.recoveryToggleBtn')}
        </Button>
      </div>
    </div>
  );
};
