import React from 'react';
import { Globe } from 'lucide-react';
import { Button, ButtonProps } from './button';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface LanguageToggleButtonProps
  extends Omit<ButtonProps, 'onClick' | 'children'> {}

export const LanguageToggleButton: React.FC<LanguageToggleButtonProps> = ({
  variant = 'secondary',
  size = 'md',
  className,
  ref,
  ...props
}) => {
  const { locale, setLocale, t } = useI18n();

  return (
    <Button
      ref={ref}
      variant={variant}
      size={size}
      aria-label={t('common.language')}
      title={t('common.language')}
      leftIcon={<Globe className="h-3.5 w-3.5" />}
      onClick={() => setLocale(locale === 'en' ? 'ru' : 'en')}
      className={cn('px-2.5 text-xs font-semibold text-secondary hover:text-primary', className)}
      {...props}
    >
      {locale.toUpperCase()}
    </Button>
  );
};
