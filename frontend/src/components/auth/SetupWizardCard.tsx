import React, { useEffect, useRef, useState } from 'react';
import { Fingerprint, ShieldCheck } from 'lucide-react';
import {
  Alert,
  Button,
  EmailInput,
  PasswordInput,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import {
  requestRegisterChallenge,
  submitInitialSetup,
} from '../../api/client';
import {
  createPasskeyCredential,
  isSecureContextAvailable,
} from '../../lib/webauthn';
import { useAuth } from '../../lib/auth-context';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { RecoveryCodesCard } from './RecoveryCodesCard';

export interface SetupWizardCardProps {
  defaultTreeTitle?: string;
  initialRecoveryCodes?: string[];
  onComplete?: (treeTitle?: string) => void;
}

export const SetupWizardCard: React.FC<SetupWizardCardProps> = ({
  defaultTreeTitle = 'Levshits Family Archive',
  initialRecoveryCodes,
  onComplete,
}) => {
  const { t } = useI18n();
  const { establishSession } = useAuth();

  const [treeTitle, setTreeTitle] = useState(defaultTreeTitle);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [recoveryCodes, setRecoveryCodes] = useState<string[] | null>(
    initialRecoveryCodes ?? null
  );

  const errorRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (error && errorRef.current) {
      errorRef.current.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      errorRef.current.focus();
    }
  }, [error]);

  const secureContext = isSecureContextAvailable();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (submitting) return;
    setError(null);
    setSubmitting(true);

    try {
      let passkeyPayload = undefined;
      if (secureContext) {
        const challenge = await requestRegisterChallenge();
        passkeyPayload = await createPasskeyCredential(
          challenge,
          email.trim(),
          `${firstName.trim()} ${lastName.trim()}`.trim()
        );
      }

      const res = await submitInitialSetup({
        tree_title: treeTitle.trim(),
        first_name: firstName.trim(),
        last_name: lastName.trim(),
        email: email.trim(),
        password: password.trim().length >= 10 ? password : undefined,
        passkey: passkeyPayload,
      });

      establishSession(res.user, res.session_token, false);
      setRecoveryCodes(res.recovery_codes);
    } catch (err) {
      setError(getLocalizedErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  if (recoveryCodes && recoveryCodes.length > 0) {
    return (
      <RecoveryCodesCard
        codes={recoveryCodes}
        requireConfirmation
        onContinue={() => onComplete?.(treeTitle.trim())}
      />
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {!secureContext && (
        <Alert variant="warning" title={t('auth.insecureContextTitle')}>
          {t('auth.insecureContextDesc')}
        </Alert>
      )}

      {error && (
        <Alert
          ref={errorRef}
          variant="danger"
          tabIndex={-1}
          className="focus:outline-none"
        >
          {error}
        </Alert>
      )}

      <FormField
        name="tree_title"
        label={t('auth.setup.treeTitleLabel')}
        required
      >
        <TextInput
          value={treeTitle}
          onChange={(e) => setTreeTitle(e.target.value)}
          required
        />
      </FormField>

      <FormField
        name="first_name"
        label={t('auth.setup.firstNameLabel')}
        required
      >
        <TextInput
          value={firstName}
          onChange={(e) => setFirstName(e.target.value)}
          autoComplete="given-name"
          required
        />
      </FormField>

      <FormField
        name="last_name"
        label={t('auth.setup.lastNameLabel')}
        required
      >
        <TextInput
          value={lastName}
          onChange={(e) => setLastName(e.target.value)}
          autoComplete="family-name"
          required
        />
      </FormField>

      <FormField
        name="email"
        label={t('auth.setup.emailLabel')}
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
        name="password"
        label={t('auth.setup.passwordOptionalLabel')}
        optionalText={secureContext}
        required={!secureContext}
        hint={t('auth.setup.passwordOptionalHint')}
      >
        <PasswordInput
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          showStrengthHint
        />
      </FormField>

      <div className="pt-2">
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
            !email.trim() ||
            (!secureContext && password.trim().length < 10)
          }
          leftIcon={
            secureContext ? (
              <Fingerprint className="h-4.5 w-4.5" />
            ) : (
              <ShieldCheck className="h-4.5 w-4.5" />
            )
          }
        >
          {secureContext
            ? t('auth.setup.createAdminPasskeyBtn')
            : t('auth.setup.completeSetupBtn')}
        </Button>
      </div>
    </form>
  );
};
