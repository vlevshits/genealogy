import React, { useState } from 'react';
import { LogIn, Shield, X } from 'lucide-react';
import { Alert, Badge, Button, IconButton } from '../ui';
import { useI18n } from '../../i18n';

export interface PublicGuestBannerProps {
  onSignIn?: () => void;
  onDismiss?: () => void;
  className?: string;
}

export const PublicGuestBanner: React.FC<PublicGuestBannerProps> = ({
  onSignIn,
  onDismiss,
  className,
}) => {
  const { t } = useI18n();
  const [dismissed, setDismissed] = useState<boolean>(() => {
    try {
      if (typeof window !== 'undefined' && window.sessionStorage) {
        return window.sessionStorage.getItem('genealogy_dismiss_public_banner') === 'true';
      }
    } catch {
      // ignore storage access errors
    }
    return false;
  });

  if (dismissed) return null;

  const handleDismiss = () => {
    setDismissed(true);
    try {
      if (typeof window !== 'undefined' && window.sessionStorage) {
        window.sessionStorage.setItem('genealogy_dismiss_public_banner', 'true');
      }
    } catch {
      // ignore storage access errors
    }
    onDismiss?.();
  };

  return (
    <Alert
      variant="info"
      className={className}
      icon={<Shield className="h-4.5 w-4.5 text-brand shrink-0 mt-0.5" />}
      title={t('tree.publicBanner.title')}
      action={
        <div className="flex items-center gap-2">
          {onSignIn && (
            <Button
              size="sm"
              variant="primary"
              leftIcon={<LogIn className="h-3.5 w-3.5" />}
              onClick={onSignIn}
              className="hidden sm:inline-flex"
            >
              {t('tree.publicBanner.signInBtn')}
            </Button>
          )}
          <IconButton
            size="sm"
            variant="ghost"
            label={t('common.close')}
            onClick={handleDismiss}
            className="min-h-10 min-w-10 text-secondary hover:text-primary"
          >
            <X className="h-4 w-4" />
          </IconButton>
        </div>
      }
    >
      <div className="space-y-2">
        <div>
          <Badge variant="brand" size="sm">
            {t('tree.publicBanner.badge')}
          </Badge>
        </div>
        <p className="text-xs sm:text-sm text-secondary leading-relaxed">
          {t('tree.publicBanner.desc')}
        </p>
        {onSignIn && (
          <div className="pt-1.5 sm:hidden">
            <Button
              size="sm"
              variant="primary"
              leftIcon={<LogIn className="h-3.5 w-3.5" />}
              onClick={onSignIn}
              className="w-full justify-center"
            >
              {t('tree.publicBanner.signInBtn')}
            </Button>
          </div>
        )}
      </div>
    </Alert>
  );
};
