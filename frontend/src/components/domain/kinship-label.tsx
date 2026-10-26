import React from 'react';
import { Users } from 'lucide-react';
import { Badge } from '../ui/badge';
import { formatKinshipLabel, KinshipCode } from '../../lib/kinship';
import { Sex } from '../../types/common';
import { useI18n } from '../../i18n';

export interface KinshipLabelProps {
  relationship: KinshipCode;
  sex?: Sex;
  className?: string;
}

export const KinshipLabel: React.FC<KinshipLabelProps> = ({
  relationship,
  sex = 'unknown',
  className,
}) => {
  const { locale } = useI18n();
  const label = formatKinshipLabel(relationship, sex, locale);

  return (
    <Badge
      variant="accent"
      size="md"
      icon={<Users className="h-3.5 w-3.5" />}
      className={className}
    >
      {label}
    </Badge>
  );
};
