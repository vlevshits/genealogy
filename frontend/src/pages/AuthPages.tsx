import React from 'react';
import { AuthLayout } from '../components/ui';
import {
  LoginCard,
  MagicLinkVerifyCard,
  RegisterInviteCard,
  SetupWizardCard,
} from '../components/auth';
import { useAuth } from '../lib/auth-context';
import { sanitizeReturnUrl } from '../lib/navigation-utils';
import { useI18n } from '../i18n';

export interface AuthPagesProps {
  currentPath: string;
  search?: string;
  instanceTitle?: string;
  onNavigate: (path: string) => void;
  onInstanceTitleChange?: (title: string) => void;
}

export const AuthPages: React.FC<AuthPagesProps> = ({
  currentPath,
  search = typeof window !== 'undefined' ? window.location.search : '',
  instanceTitle,
  onNavigate,
  onInstanceTitleChange,
}) => {
  const { t } = useI18n();
  const { setupStatus } = useAuth();

  const params = new URLSearchParams(search);
  const rawReturnUrl = params.get('returnUrl');
  const targetUrl = sanitizeReturnUrl(rawReturnUrl, '/');

  if (currentPath.startsWith('/setup')) {
    return (
      <AuthLayout
        title={t('auth.setup.title')}
        subtitle={t('auth.setup.subtitle')}
        instanceTitle={instanceTitle}
        onNavigate={onNavigate}
      >
        <SetupWizardCard
          defaultTreeTitle={instanceTitle || setupStatus?.tree_title}
          onComplete={(nextTitle) => {
            if (nextTitle) onInstanceTitleChange?.(nextTitle);
            onNavigate(targetUrl);
          }}
        />
      </AuthLayout>
    );
  }

  if (currentPath.startsWith('/register')) {
    const inviteToken = params.get('invite_token') || params.get('token') || '';
    return (
      <AuthLayout
        title={t('auth.register.title')}
        subtitle={t('auth.register.subtitle')}
        instanceTitle={instanceTitle}
        onNavigate={onNavigate}
      >
        <RegisterInviteCard
          inviteToken={inviteToken}
          onComplete={() => onNavigate(targetUrl)}
        />
      </AuthLayout>
    );
  }

  if (currentPath.startsWith('/magic-link/verify')) {
    const token = params.get('token') || '';
    return (
      <AuthLayout
        title={t('auth.magicVerify.title')}
        instanceTitle={instanceTitle}
        onNavigate={onNavigate}
      >
        <MagicLinkVerifyCard
          token={token}
          onComplete={() => onNavigate(targetUrl)}
        />
      </AuthLayout>
    );
  }

  return (
    <AuthLayout
      title={t('auth.login.title')}
      subtitle={t('auth.login.subtitle')}
      instanceTitle={instanceTitle}
      onNavigate={onNavigate}
    >
      <LoginCard
        authPasswordEnabled={setupStatus?.auth_password_enabled ?? true}
        smtpEnabled={setupStatus?.smtp_enabled ?? false}
        enablePublicAccess={setupStatus?.enable_public_access ?? false}
        returnUrl={rawReturnUrl ?? undefined}
        onNavigate={onNavigate}
        onSuccess={() => onNavigate(targetUrl)}
      />
    </AuthLayout>
  );
};

export default AuthPages;
