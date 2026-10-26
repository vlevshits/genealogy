import React, { useEffect, useState } from 'react';
import { CheckCircle2, Fingerprint } from 'lucide-react';
import { Alert, Button, Spinner } from '../ui';
import {
  AuthSessionResponse,
  requestAddPasskeyChallenge,
  UserDto,
  verifyAddPasskey,
  verifyMagicLink,
} from '../../api/client';
import { createPasskeyCredential } from '../../lib/webauthn';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';

// Module-level deduplication cache so <React.StrictMode> double-invocation of useEffect
// does not issue a second POST /api/v1/auth/magic-link/verify against a single-use token.
const verificationPromiseByToken = new Map<string, Promise<AuthSessionResponse>>();

export function clearMagicLinkVerificationCache(): void {
  verificationPromiseByToken.clear();
}

export interface MagicLinkVerifyCardProps {
  token: string;
  initialVerifiedUser?: UserDto | null;
  onComplete?: () => void;
}

export const MagicLinkVerifyCard: React.FC<MagicLinkVerifyCardProps> = ({
  token,
  initialVerifiedUser,
  onComplete,
}) => {
  const { t } = useI18n();
  const { establishSession } = useAuth();

  const [verifiedUser, setVerifiedUser] = useState<UserDto | null>(
    initialVerifiedUser ?? null
  );
  const [verifying, setVerifying] = useState<boolean>(
    initialVerifiedUser === undefined
  );
  const [enrolling, setEnrolling] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialVerifiedUser !== undefined) return;
    if (!token) {
      setError(t('common.errorLoading'));
      setVerifying(false);
      return;
    }

    let cancelled = false;
    let verificationPromise = verificationPromiseByToken.get(token);
    if (!verificationPromise) {
      verificationPromise = verifyMagicLink({ token });
      verificationPromiseByToken.set(token, verificationPromise);
    }

    verificationPromise
      .then((res) => {
        if (cancelled) return;
        establishSession(res.user, res.session_token, res.prompt_passkey_enrollment);
        setVerifiedUser(res.user);
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : t('common.errorLoading'));
        }
      })
      .finally(() => {
        if (!cancelled) setVerifying(false);
      });

    return () => {
      cancelled = true;
    };
  }, [token, initialVerifiedUser, establishSession, t]);

  const handleEnrollPasskey = async () => {
    if (!verifiedUser || enrolling) return;
    setEnrolling(true);
    try {
      const challenge = await requestAddPasskeyChallenge();
      const cred = await createPasskeyCredential(
        challenge,
        verifiedUser.email,
        verifiedUser.display_name
      );
      await verifyAddPasskey(cred);
      onComplete?.();
    } catch {
      onComplete?.();
    } finally {
      setEnrolling(false);
    }
  };

  if (verifying) {
    return (
      <div className="flex items-center justify-center gap-2 py-8 text-sm text-secondary">
        <Spinner size="sm" />
        <span>{t('auth.magicVerify.verifying')}</span>
      </div>
    );
  }

  if (error || !verifiedUser) {
    return <Alert variant="danger">{error || t('common.errorLoading')}</Alert>;
  }

  return (
    <div className="space-y-4">
      <Alert
        variant="success"
        icon={<CheckCircle2 className="h-4 w-4 text-success shrink-0 mt-0.5" />}
        title={t('auth.magicVerify.successTitle')}
      >
        {t('auth.magicVerify.enrollPromptDesc')}
      </Alert>

      <Button
        variant="primary"
        fullWidth
        loading={enrolling}
        leftIcon={<Fingerprint className="h-4 w-4" />}
        onClick={handleEnrollPasskey}
      >
        {t('auth.magicVerify.enrollNowBtn')}
      </Button>

      <Button variant="secondary" fullWidth onClick={() => onComplete?.()}>
        {t('auth.magicVerify.skipBtn')}
      </Button>
    </div>
  );
};
