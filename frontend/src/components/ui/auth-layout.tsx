import React from 'react';
import { useI18n } from '../../i18n';
import { BrandLogoIcon } from './brand-logo-icon';
import { LanguageToggleButton } from './language-toggle-button';
import { ThemeToggleButton } from './theme-toggle-button';

export interface AuthLayoutProps {
  title: string;
  subtitle?: string;
  instanceTitle?: string;
  onNavigate?: (path: string) => void;
  onNavigateHome?: () => void;
  children: React.ReactNode;
}

export const AuthLayout: React.FC<AuthLayoutProps> = ({
  title,
  subtitle,
  instanceTitle,
  onNavigate,
  onNavigateHome,
  children,
}) => {
  const { t } = useI18n();
  const resolvedTitle = instanceTitle || t('app.instanceName');

  const handleNavigateHome = (e: React.MouseEvent<HTMLAnchorElement>) => {
    if (e.defaultPrevented || e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    if (onNavigateHome) {
      onNavigateHome();
    } else if (onNavigate) {
      onNavigate('/');
    } else if (typeof window !== 'undefined') {
      window.location.href = '/';
    }
  };

  return (
    <div className="min-h-screen flex flex-col justify-between bg-canvas text-primary p-4 sm:p-6">
      <header className="flex items-center justify-between max-w-md w-full mx-auto">
        <a
          href="/"
          onClick={handleNavigateHome}
          className="flex items-center gap-2.5 rounded-lg min-h-10 py-1 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-focus hover:opacity-90 transition-opacity"
          aria-label={resolvedTitle}
        >
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-brand text-surface shadow-xs">
            <BrandLogoIcon className="h-4.5 w-4.5" />
          </div>
          <span className="font-sans font-semibold tracking-tight text-sm text-primary whitespace-nowrap truncate max-w-[200px] sm:max-w-none">
            {resolvedTitle}
          </span>
        </a>
        <div className="flex items-center gap-2">
          <LanguageToggleButton size="md" />
          <ThemeToggleButton size="md" />
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
