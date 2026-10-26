import React from 'react';
import { LogIn, Shield } from 'lucide-react';
import { Alert, Badge, Button } from '../ui';
import { useI18n } from '../../i18n';

export interface PublicGuestBannerProps {
  onSignIn?: () => void;
  className?: string;
}

export const PublicGuestBanner: React.FC<PublicGuestBannerProps> = ({
  onSignIn,
  className,
}) => {
  const { t } = useI18n();

  return (
    <Alert
      variant="info"
      className={className}
      icon={<Shield className="h-4.5 w-4.5 text-brand shrink-0 mt-0.5" />}
      title={t('tree.publicBanner.title')}
      action={
        onSignIn ? (
          <Button
            size="sm"
            variant="primary"
            leftIcon={<LogIn className="h-3.5 w-3.5" />}
            onClick={onSignIn}
          >
            {t('tree.publicBanner.signInBtn')}
          </Button>
        ) : undefined
      }
    >
      <div className="space-y-1.5">
        <div>
          <Badge variant="brand" size="sm">
            {t('tree.publicBanner.badge')}
          </Badge>
        </div>
        <p className="text-xs sm:text-sm text-secondary leading-relaxed">
          {t('tree.publicBanner.desc')}
        </p>
      </div>
    </Alert>
  );
};
