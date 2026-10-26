import React from 'react';
import { GitBranch, Globe, Moon, Sun } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useTheme } from '../../lib/theme';

export interface AuthLayoutProps {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
}

export const AuthLayout: React.FC<AuthLayoutProps> = ({ title, subtitle, children }) => {
  const { locale, setLocale, t } = useI18n();
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <div className="min-h-screen flex flex-col justify-between bg-canvas text-primary p-4 sm:p-6">
      <div className="flex items-center justify-between max-w-md w-full mx-auto">
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-brand text-surface">
            <GitBranch className="h-4 w-4" />
          </div>
          <span className="font-serif font-semibold text-sm">{t('app.instanceName')}</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setLocale(locale === 'en' ? 'ru' : 'en')}
            className="inline-flex items-center gap-1 rounded-lg border border-border bg-surface px-2.5 py-1 text-xs font-medium text-secondary hover:text-primary cursor-pointer"
          >
            <Globe className="h-3.5 w-3.5" />
            <span>{locale.toUpperCase()}</span>
          </button>
          <button
            type="button"
            aria-label={t('common.themeSelector')}
            onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
            className="rounded-lg border border-border bg-surface p-1.5 text-secondary hover:text-primary cursor-pointer"
          >
            {resolvedTheme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </button>
        </div>
      </div>

      <div className="my-auto max-w-md w-full mx-auto rounded-2xl border border-border bg-surface p-6 sm:p-8 shadow-lg space-y-6">
        <div className="text-center space-y-1">
          <h1 className="font-serif text-2xl font-bold text-primary">{title}</h1>
          {subtitle && <p className="text-xs sm:text-sm text-secondary">{subtitle}</p>}
        </div>
        {children}
      </div>

      <div className="text-center text-xs text-tertiary">{t('app.footerCopyright')}</div>
    </div>
  );
};
