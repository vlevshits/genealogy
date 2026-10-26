import React from 'react';
import { Laptop, Moon, Sun } from 'lucide-react';
import { IconButton, IconButtonProps } from './icon-button';
import { useTheme } from '../../lib/theme';
import { useI18n } from '../../i18n';
import { ThemeMode } from '../../types/common';

export interface ThemeToggleButtonProps
  extends Omit<IconButtonProps, 'label' | 'onClick' | 'children'> {
  label?: string;
}

export const ThemeToggleButton: React.FC<ThemeToggleButtonProps> = ({
  label,
  variant = 'secondary',
  size = 'md',
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const { theme, setTheme } = useTheme();

  const cycleTheme = () => {
    const order: ThemeMode[] = ['light', 'dark', 'system'];
    const next = order[(order.indexOf(theme) + 1) % order.length];
    setTheme(next);
  };

  const modeLabel =
    theme === 'light'
      ? t('common.lightMode')
      : theme === 'dark'
        ? t('common.darkMode')
        : t('common.systemMode');

  return (
    <IconButton
      ref={ref}
      variant={variant}
      size={size}
      label={label ?? `${t('common.themeSelector')}: ${modeLabel}`}
      onClick={cycleTheme}
      {...props}
    >
      {theme === 'light' && <Sun className="h-4 w-4" />}
      {theme === 'dark' && <Moon className="h-4 w-4" />}
      {theme === 'system' && <Laptop className="h-4 w-4" />}
    </IconButton>
  );
};
