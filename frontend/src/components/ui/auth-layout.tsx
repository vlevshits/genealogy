import React from 'react';
import { useI18n } from '../../i18n';
import { BrandLogoIcon } from './brand-logo-icon';
import { LanguageToggleButton } from './language-toggle-button';
import { ThemeToggleButton } from './theme-toggle-button';

export interface AuthLayoutProps {
  title: string;
  subtitle?: string;
  instanceTitle?: string;
  children: React.ReactNode;
}

export const AuthLayout: React.FC<AuthLayoutProps> = ({
  title,
  subtitle,
  instanceTitle,
  children,
}) => {
  const { t } = useI18n();
  const resolvedTitle = instanceTitle || t('app.instanceName');

  return (
    <div className="min-h-screen flex flex-col justify-between bg-canvas text-primary p-4 sm:p-6">
      <header className="flex items-center justify-between max-w-md w-full mx-auto">
        <div className="flex items-center gap-2.5">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-brand text-surface shadow-xs">
            <BrandLogoIcon className="h-4.5 w-4.5" />
          </div>
          <span className="font-sans font-semibold tracking-tight text-sm text-primary">
            {resolvedTitle}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <LanguageToggleButton size="sm" />
          <ThemeToggleButton size="sm" />
        </div>
      </header>

      <main className="my-auto max-w-md w-full mx-auto rounded-2xl border border-border bg-surface p-6 sm:p-8 shadow-lg space-y-6">
        <div className="text-center space-y-1">
          <h1 className="font-sans text-2xl font-semibold tracking-tight text-primary">
            {title}
          </h1>
          {subtitle && <p className="text-xs sm:text-sm text-secondary">{subtitle}</p>}
        </div>
        {children}
      </main>

      <footer className="text-center text-xs text-tertiary">
        {t('app.footerCopyright')} · {t('app.footerRight')}
      </footer>
    </div>
  );
};
