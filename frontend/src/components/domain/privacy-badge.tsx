import React from 'react';
import { Lock, Shield } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Tooltip } from '../ui/tooltip';
import { useI18n } from '../../i18n';

export const PrivacyBadge: React.FC<{ className?: string }> = ({ className }) => {
  const { t } = useI18n();
  return (
    <Tooltip content={t('domain.privateTooltip')}>
      <Badge
        variant="warning"
        size="sm"
        icon={<Lock className="h-3 w-3" />}
        className={className}
      >
        {t('domain.privateBadge')}
      </Badge>
    </Tooltip>
  );
};

export const LivingBadge: React.FC<{ className?: string }> = ({ className }) => {
  const { t } = useI18n();
  return (
    <Tooltip content={t('domain.livingTooltip')}>
      <Badge
        variant="brand"
        size="sm"
        icon={<Shield className="h-3 w-3" />}
        className={className}
      >
        {t('domain.living')}
      </Badge>
    </Tooltip>
  );
};
