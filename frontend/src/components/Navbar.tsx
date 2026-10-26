import React, { useState, useEffect } from 'react';
import { GitBranch, Moon, Sun, Globe, CheckCircle2, AlertTriangle, RefreshCw } from 'lucide-react';
import { useI18n } from '../i18n';
import { fetchHealth, HealthResponse } from '../api/client';

export const Navbar: React.FC = () => {
  const { t, locale, setLocale } = useI18n();
  const [isDark, setIsDark] = useState<boolean>(() => {
    return document.documentElement.classList.contains('dark') ||
      (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches);
  });

  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }, [isDark]);

  useEffect(() => {
    let mounted = true;
    const checkStatus = () => {
      fetchHealth()
        .then((data) => {
          if (mounted) {
            setHealth(data);
            setLoading(false);
          }
        })
        .catch(() => {
          if (mounted) {
            setHealth(null);
            setLoading(false);
          }
        });
    };

    checkStatus();
    const interval = setInterval(checkStatus, 15000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  return (
    <header className="sticky top-0 z-50 w-full border-b border-border bg-surface/85 backdrop-blur-md transition-colors duration-200">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        {/* Brand & Logo */}
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-brand text-white shadow-sm transition-transform hover:scale-105">
            <GitBranch className="h-5 w-5" />
          </div>
          <div>
            <span className="font-serif text-lg font-semibold tracking-tight text-primary">
              {t('app.title')}
            </span>
            <span className="hidden text-xs text-secondary sm:inline-block sm:ml-2">
              v0.1.0
            </span>
          </div>
        </div>

        {/* Navigation Links */}
        <nav className="hidden md:flex items-center gap-1 text-sm font-medium text-secondary">
          <span className="cursor-pointer rounded-lg px-3 py-1.5 text-primary hover:bg-surface-muted transition-colors">
            {t('nav.tree')}
          </span>
          <span className="cursor-pointer rounded-lg px-3 py-1.5 hover:bg-surface-muted hover:text-primary transition-colors">
            {t('nav.directory')}
          </span>
          <span className="cursor-pointer rounded-lg px-3 py-1.5 hover:bg-surface-muted hover:text-primary transition-colors">
            {t('nav.places')}
          </span>
          <span className="cursor-pointer rounded-lg px-3 py-1.5 hover:bg-surface-muted hover:text-primary transition-colors">
            {t('nav.media')}
          </span>
          <span className="cursor-pointer rounded-lg px-3 py-1.5 hover:bg-surface-muted hover:text-primary transition-colors">
            {t('nav.settings')}
          </span>
        </nav>

        {/* Right side controls (Locale, Theme, Status) */}
        <div className="flex items-center gap-2">
          {/* Health Badge */}
          <div className="flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium border border-border bg-surface-muted">
            {loading ? (
              <RefreshCw className="h-3 w-3 animate-spin text-tertiary" />
            ) : health ? (
              <>
                <CheckCircle2 className="h-3 w-3 text-success" />
                <span className="text-secondary">{t('status.healthy')}</span>
              </>
            ) : (
              <>
                <AlertTriangle className="h-3 w-3 text-warning" />
                <span className="text-secondary">{t('status.degraded')}</span>
              </>
            )}
          </div>

          {/* Locale Toggle */}
          <button
            onClick={() => setLocale(locale === 'en' ? 'ru' : 'en')}
            className="flex items-center gap-1 rounded-lg border border-border px-2.5 py-1.5 text-xs font-semibold text-secondary hover:bg-surface-muted hover:text-primary transition-colors"
            title={t('common.language')}
          >
            <Globe className="h-3.5 w-3.5" />
            <span>{locale.toUpperCase()}</span>
          </button>

          {/* Dark Mode Toggle */}
          <button
            onClick={() => setIsDark(!isDark)}
            className="rounded-lg border border-border p-2 text-secondary hover:bg-surface-muted hover:text-primary transition-colors"
            title={isDark ? t('common.lightMode') : t('common.darkMode')}
          >
            {isDark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </button>
        </div>
      </div>
    </header>
  );
};
