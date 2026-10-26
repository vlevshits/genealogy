import React from 'react';
import { Shield, PenTool, Eye, Ban } from 'lucide-react';
import { Badge, BadgeProps } from './badge';
import { useI18n } from '../../i18n';

export interface RoleBadgeProps {
  role: 'admin' | 'editor' | 'viewer' | string;
  isActive?: boolean;
  size?: BadgeProps['size'];
  showIcon?: boolean;
  className?: string;
}

export const RoleBadge: React.FC<RoleBadgeProps> = ({
  role,
  isActive = true,
  size = 'sm',
  showIcon = false,
  className,
}) => {
  const { t } = useI18n();

  const renderRolePill = () => {
    if (role === 'admin') {
      return (
        <Badge
          variant="brand"
          size={size}
          icon={showIcon ? <Shield className="h-3 w-3" /> : undefined}
          className={className}
        >
          {t('settings.users.roleAdmin')}
        </Badge>
      );
    }
    if (role === 'editor') {
      return (
        <Badge
          variant="success"
          size={size}
          icon={showIcon ? <PenTool className="h-3 w-3" /> : undefined}
          className={className}
        >
          {t('settings.users.roleEditor')}
        </Badge>
      );
    }
    return (
      <Badge
        variant="neutral"
        size={size}
        icon={showIcon ? <Eye className="h-3 w-3" /> : undefined}
        className={className}
      >
        {t('settings.users.roleViewer')}
      </Badge>
    );
  };

  if (!isActive) {
    return (
      <span className="inline-flex flex-wrap items-center gap-1.5">
        {renderRolePill()}
        <Badge
          variant="danger"
          size={size}
          icon={<Ban className="h-3 w-3" />}
        >
          {t('settings.users.statusDeactivated')}
        </Badge>
      </span>
    );
  }

  return renderRolePill();
};
