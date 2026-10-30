import React, { useState, useRef, useEffect } from 'react';
import {
  ArrowLeft,
  Compass,
  Fingerprint,
  KeyRound,
  Lock,
  Mail,
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
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { resolveGuestEgressTarget } from '../../lib/navigation-utils';
import { useI18n } from '../../i18n';

export type LoginViewMode = 'default' | 'magic_sent' | 'recovery';

export interface LoginCardProps {
  authPasswordEnabled?: boolean;
  smtpEnabled?: boolean;
  enablePublicAccess?: boolean;
  returnUrl?: string;
  initialMode?: LoginViewMode;
  onSuccess?: (promptPasskeyEnrollment?: boolean) => void;
  onNavigate?: (path: string) => void;
}

export const LoginCard: React.FC<LoginCardProps> = ({
  authPasswordEnabled = true,
  smtpEnabled: smtpEnabledProp,
  enablePublicAccess: enablePublicAccessProp,
  returnUrl,
  initialMode = 'default',
  onSuccess,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { establishSession, setupStatus } = useAuth();
  const smtpEnabled = smtpEnabledProp ?? setupStatus?.smtp_enabled ?? false;
  const enablePublicAccess =
    enablePublicAccessProp ?? setupStatus?.enable_public_access ?? false;

  const egressTarget = resolveGuestEgressTarget(returnUrl);
  const isTreeOrProfileTarget =
    egressTarget.startsWith('/tree') || egressTarget.startsWith('/persons/');
  const egressLabel = isTreeOrProfileTarget
    ? t('auth.login.backToTreeBtn')
    : t('auth.login.browsePublicBtn');

  const [mode, setMode] = useState<LoginViewMode>(initialMode);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [recoveryCode, setRecoveryCode] = useState('');
  const [loadingPasskey, setLoadingPasskey] = useState(false);
  const [loadingPassword, setLoadingPassword] = useState(false);
  const [loadingMagic, setLoadingMagic] = useState(false);
  const [loadingRecovery, setLoadingRecovery] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const emailInputRef = useRef<HTMLInputElement>(null);
  const passkeyButtonRef = useRef<HTMLButtonElement>(null);
  const backToSignInRef = useRef<HTMLButtonElement>(null);
  const restoreFocusToRef = useRef<'email' | 'passkey' | null>(null);

  useEffect(() => {
    if (mode === 'magic_sent') {
      backToSignInRef.current?.focus();
    }
  }, [mode]);

  useEffect(() => {
    if (mode === 'default' && restoreFocusToRef.current) {
      if (restoreFocusToRef.current === 'email' && emailInputRef.current) {
        emailInputRef.current.focus();
      } else if (restoreFocusToRef.current === 'passkey' && passkeyButtonRef.current) {
        passkeyButtonRef.current.focus();
      }
      restoreFocusToRef.current = null;
    }
  }, [mode]);

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
      setError(getLocalizedErrorMessage(err, t));
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
      setError(getLocalizedErrorMessage(err, t));
    } finally {
      setLoadingPassword(false);
    }
  };

  const handleRequestMagicLink = async () => {
    if (!smtpEnabled || loadingMagic || !email.trim()) return;
    setError(null);
    setLoadingMagic(true);
    try {
      await requestMagicLink({ email: email.trim() });
      setMode('magic_sent');
    } catch (err) {
      setError(getLocalizedErrorMessage(err, t));
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
      setError(getLocalizedErrorMessage(err, t));
    } finally {
      setLoadingRecovery(false);
    }
  };

  if (mode === 'magic_sent') {
    const targetEmail = email.trim() || 'user@example.com';
    const descTemplate = t('auth.login.magicLinkSentDesc', {
      email: '___EMAIL_SLOT___',
    });
    const [descBefore, descAfter] = descTemplate.split('___EMAIL_SLOT___');

    return (
      <div className="space-y-4">
        <Alert
          variant="success"
          icon={<Mail className="h-4 w-4 text-success shrink-0 mt-0.5" />}
          title={t('auth.login.magicLinkSentTitle')}
        >
          <p className="break-words">
            {descBefore}
            <span className="font-medium text-primary break-all">
              {targetEmail}
            </span>
            {descAfter}
          </p>
        </Alert>

        <Button
          ref={backToSignInRef}
          autoFocus
          variant="secondary"
          fullWidth
          onClick={() => {
            restoreFocusToRef.current = 'email';
            setMode('default');
          }}
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
            restoreFocusToRef.current = 'passkey';
            setMode('default');
          }}
        >
          {t('auth.login.backToLoginBtn')}
        </Button>
      </form>
    );
  }

  const showDivider = authPasswordEnabled || smtpEnabled;
  const dividerLabel =
    authPasswordEnabled && smtpEnabled
      ? t('auth.login.orPasswordOrEmailDivider')
      : authPasswordEnabled
        ? t('auth.login.orPasswordDivider')
        : t('auth.login.orEmailDivider');

  return (
    <div className="space-y-5">
      {!secureContext ? (
        <Alert variant="warning" title={t('auth.insecureContextTitle')}>
          {t('auth.insecureContextDesc')}
        </Alert>
      ) : (
        <div className="space-y-1.5">
          <Button
            ref={passkeyButtonRef}
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

      {showDivider && <Divider label={dividerLabel} />}

      {(authPasswordEnabled || smtpEnabled) ? (
        <form onSubmit={handlePasswordSignIn} className="space-y-3">
          <FormField
            name="email"
            label={t('auth.login.emailLabel')}
            required
          >
            <EmailInput
              ref={emailInputRef}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('auth.login.emailPlaceholder')}
              autoComplete="username webauthn"
              aria-describedby={!smtpEnabled ? 'smtp-disabled-hint' : undefined}
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

            {smtpEnabled && (
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
            )}

            {!smtpEnabled && (
              <Alert
                id="smtp-disabled-hint"
                variant="info"
                role="status"
                tabIndex={0}
                className="text-xs p-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
              >
                {t('auth.login.smtpDisabledHint')}
              </Alert>
            )}
          </div>
        </form>
      ) : (
        <Alert
          id="smtp-disabled-hint"
          variant="info"
          role="status"
          tabIndex={0}
          className="text-xs p-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
        >
          {t('auth.login.smtpDisabledNoPasswordHint')}
        </Alert>
      )}

      {enablePublicAccess && (
        <div className="pt-1">
          <Button
            type="button"
            variant="secondary"
            fullWidth
            leftIcon={
              isTreeOrProfileTarget ? (
                <ArrowLeft className="h-4 w-4" />
              ) : (
                <Compass className="h-4 w-4" />
              )
            }
            onClick={() => {
              if (onNavigate) {
                onNavigate(egressTarget);
              } else if (typeof window !== 'undefined') {
                window.location.href = egressTarget;
              }
            }}
          >
            {egressLabel}
          </Button>
        </div>
      )}

      <div className="pt-2 flex justify-center text-center">
        <Button
          variant="link"
          size="sm"
          className="inline-flex items-start text-left leading-snug whitespace-normal max-w-full"
          leftIcon={<KeyRound className="h-3.5 w-3.5 shrink-0 mt-0.5" />}
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
