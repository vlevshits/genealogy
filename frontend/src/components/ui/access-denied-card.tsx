import React from 'react';
import { ShieldAlert, User, ArrowLeft } from 'lucide-react';
import { Badge } from './badge';
import { Button } from './button';
import { Card, CardBody, CardFooter, CardHeader } from './card';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface AccessDeniedCardProps {
  title?: string;
  description?: string;
  currentRole?: 'admin' | 'editor' | 'viewer' | string;
  onOpenProfile?: () => void;
  onBackToTree?: () => void;
  className?: string;
}

export const AccessDeniedCard: React.FC<AccessDeniedCardProps> = ({
  title,
  description,
  onOpenProfile,
  onBackToTree,
  className,
}) => {
  const { t } = useI18n();

  return (
    <Card
      role="alert"
      className={cn('max-w-xl mx-auto border-warning/40', className)}
    >
      <CardHeader className="flex flex-wrap items-start justify-between gap-2 sm:gap-3">
        <div className="flex min-w-0 items-center gap-2.5">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-warning/15 text-warning">
            <ShieldAlert className="h-5 w-5" />
          </div>
          <div className="min-w-0">
            <h2 className="font-sans text-base sm:text-lg font-semibold tracking-tight text-primary">
              {title || t('rbac.accessDeniedTitle')}
            </h2>
          </div>
        </div>
        <Badge variant="warning" size="sm" className="shrink-0 whitespace-nowrap">
          {t('rbac.adminOnlyBadge')}
        </Badge>
      </CardHeader>

      <CardBody>
        <p className="text-sm text-secondary leading-relaxed">
          {description || t('rbac.accessDeniedDesc')}
        </p>
      </CardBody>

      {(onOpenProfile || onBackToTree) && (
        <CardFooter className="flex flex-wrap items-center gap-2">
          {onOpenProfile && (
            <Button
              variant="primary"
              size="md"
              leftIcon={<User className="h-4 w-4" />}
              onClick={onOpenProfile}
            >
              {t('rbac.openMyProfileBtn')}
            </Button>
          )}
          {onBackToTree && (
            <Button
              variant="secondary"
              size="md"
              leftIcon={<ArrowLeft className="h-4 w-4" />}
              onClick={onBackToTree}
            >
              {t('rbac.backToTreeBtn')}
            </Button>
          )}
        </CardFooter>
      )}
    </Card>
  );
};
