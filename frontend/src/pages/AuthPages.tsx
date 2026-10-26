import React from 'react';
import { AuthLayout } from '../components/ui';
import {
  LoginCard,
  MagicLinkVerifyCard,
  RegisterInviteCard,
  SetupWizardCard,
} from '../components/auth';
import { useAuth } from '../lib/auth-context';
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

  if (currentPath.startsWith('/setup')) {
    return (
      <AuthLayout
        title={t('auth.setup.title')}
        subtitle={t('auth.setup.subtitle')}
        instanceTitle={instanceTitle}
      >
        <SetupWizardCard
          defaultTreeTitle={instanceTitle || setupStatus?.tree_title}
          onComplete={(nextTitle) => {
            if (nextTitle) onInstanceTitleChange?.(nextTitle);
            onNavigate('/');
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
      >
        <RegisterInviteCard
          inviteToken={inviteToken}
          onComplete={() => onNavigate('/')}
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
      >
        <MagicLinkVerifyCard
          token={token}
          onComplete={() => onNavigate('/')}
        />
      </AuthLayout>
    );
  }

  return (
    <AuthLayout
      title={t('auth.login.title')}
      subtitle={t('auth.login.subtitle')}
      instanceTitle={instanceTitle}
    >
      <LoginCard
        authPasswordEnabled={setupStatus?.auth_password_enabled ?? true}
        onNavigate={onNavigate}
        onSuccess={() => onNavigate('/')}
      />
    </AuthLayout>
  );
};

export default AuthPages;
