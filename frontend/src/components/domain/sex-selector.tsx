import React from 'react';
import { SegmentedControl } from '../ui/segmented-control';
import { Sex } from '../../types/common';
import { useI18n } from '../../i18n';

export interface SexSelectorProps {
  value?: Sex;
  defaultValue?: Sex;
  onChange?: (sex: Sex) => void;
  disabled?: boolean;
  className?: string;
}

export const SexSelector: React.FC<SexSelectorProps> = ({
  value,
  defaultValue = 'unknown',
  onChange,
  disabled = false,
  className,
}) => {
  const { t } = useI18n();

  return (
    <SegmentedControl<Sex>
      value={value}
      defaultValue={defaultValue}
      onChange={onChange}
      disabled={disabled}
      ariaLabel="Sex selector"
      className={className}
      options={[
        {
          value: 'male',
          label: t('domain.sex.male'),
          icon: <span className="h-2.5 w-2.5 rounded-full bg-male" />,
          accentClassName: 'ring-1 ring-male/50',
        },
        {
          value: 'female',
          label: t('domain.sex.female'),
          icon: <span className="h-2.5 w-2.5 rounded-full bg-female" />,
          accentClassName: 'ring-1 ring-female/50',
        },
        {
          value: 'other',
          label: t('domain.sex.other'),
          icon: <span className="h-2.5 w-2.5 rounded-full bg-accent" />,
        },
        {
          value: 'unknown',
          label: t('domain.sex.unknown'),
          icon: <span className="h-2.5 w-2.5 rounded-full bg-neutral" />,
        },
      ]}
    />
  );
};
