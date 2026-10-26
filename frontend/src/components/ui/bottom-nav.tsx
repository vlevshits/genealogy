import React, { useState } from 'react';
import {
  Activity,
  GitFork,
  Globe,
  Home,
  Image as ImageIcon,
  MapPin,
  MoreHorizontal,
  Printer,
  Settings,
  ShieldCheck,
  Sun,
  Moon,
  Monitor,
  Users,
} from 'lucide-react';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';
import { Drawer } from './drawer';
import { ThemeMode } from '../../types/common';

export interface BottomNavProps {
  currentPath?: string;
  onNavigate?: (path: string) => void;
  themeMode?: ThemeMode;
  onThemeChange?: (mode: ThemeMode) => void;
  /** When true, renders statically inside a container (used by /dev/ui workbench) instead of fixed to viewport bottom */
  inlinePreview?: boolean;
  className?: string;
}

export const BottomNav: React.FC<BottomNavProps> = ({
  currentPath = '/',
  onNavigate,
  themeMode = 'system',
  onThemeChange,
  inlinePreview = false,
  className,
}) => {
  const { locale, setLocale, t } = useI18n();
  const [moreOpen, setMoreOpen] = useState(false);

  const handleNav = (e: React.MouseEvent<HTMLAnchorElement>, href: string) => {
    if (onNavigate) {
      e.preventDefault();
      onNavigate(href);
    }
    setMoreOpen(false);
  };

  const primaryTabs = [
    {
      id: 'home',
      label: t('nav.home'),
      href: '/',
      icon: <Home className="h-5 w-5" />,
    },
    {
      id: 'tree',
      label: t('nav.tree'),
      href: '/tree',
      icon: <GitFork className="h-5 w-5" />,
    },
    {
      id: 'people',
      label: t('nav.people'),
      href: '/people',
      icon: <Users className="h-5 w-5" />,
    },
    {
      id: 'media',
      label: t('nav.media'),
      href: '/media',
      icon: <ImageIcon className="h-5 w-5" />,
    },
  ];

  const moreLinks = [
    {
      id: 'places',
      label: t('nav.places'),
      href: '/places',
      icon: <MapPin className="h-4 w-4" />,
    },
    {
      id: 'print',
      label: t('nav.print'),
      href: '/export/print',
      icon: <Printer className="h-4 w-4" />,
    },
    {
      id: 'changes',
      label: t('nav.history'),
      href: '/history',
      icon: <Activity className="h-4 w-4" />,
    },
    {
      id: 'quality',
      label: t('nav.quality'),
      href: '/quality',
      icon: <ShieldCheck className="h-4 w-4" />,
    },
    {
      id: 'admin',
      label: t('nav.adminSettings'),
      href: '/settings',
      icon: <Settings className="h-4 w-4" />,
    },
  ];

  const isMoreActive = moreLinks.some((item) => item.href === currentPath);

  return (
    <>
      <nav
        aria-label={t('nav.openMenu')}
        className={cn(
          'border-t border-border bg-surface/95 backdrop-blur-md',
          inlinePreview
            ? 'relative w-full rounded-xl border shadow-sm'
            : 'fixed inset-x-0 bottom-0 z-30 md:hidden',
          'min-h-16 pb-[max(0.5rem,env(safe-area-inset-bottom,0px))]',
          className
        )}
      >
        <div className="mx-auto grid max-w-md grid-cols-5 items-center px-1 pt-1">
          {primaryTabs.map((tab) => {
            const active = currentPath === tab.href;
            return (
              <a
                key={tab.id}
                href={tab.href}
                onClick={(e) => handleNav(e, tab.href)}
                aria-current={active ? 'page' : undefined}
                className={cn(
                  'flex min-h-12 min-w-12 flex-col items-center justify-center gap-0.5 rounded-lg px-1 py-1 text-[11px] font-medium transition-colors',
                  active
                    ? 'text-brand font-semibold'
                    : 'text-secondary hover:bg-surface-muted hover:text-primary'
                )}
              >
                {tab.icon}
                <span className="truncate max-w-full">{tab.label}</span>
              </a>
            );
          })}

          <button
            type="button"
            aria-expanded={moreOpen}
            aria-haspopup="dialog"
            onClick={() => setMoreOpen(true)}
            className={cn(
              'flex min-h-12 min-w-12 flex-col items-center justify-center gap-0.5 rounded-lg px-1 py-1 text-[11px] font-medium transition-colors cursor-pointer',
              moreOpen || isMoreActive
                ? 'text-brand font-semibold'
                : 'text-secondary hover:bg-surface-muted hover:text-primary'
            )}
          >
            <MoreHorizontal className="h-5 w-5" />
            <span className="truncate max-w-full">{t('nav.more')}</span>
          </button>
        </div>
      </nav>

      <Drawer
        open={moreOpen}
        onOpenChange={setMoreOpen}
        side="bottom"
        title={t('nav.more')}
      >
        <div className="space-y-4">
          <div className="grid grid-cols-1 gap-1">
            {moreLinks.map((item) => {
              const active = currentPath === item.href;
              return (
                <a
                  key={item.id}
                  href={item.href}
                  onClick={(e) => handleNav(e, item.href)}
                  className={cn(
                    'flex min-h-11 items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors',
                    active
                      ? 'bg-brand/10 text-brand'
                      : 'text-primary hover:bg-surface-muted'
                  )}
                >
                  <span className="text-secondary">{item.icon}</span>
                  <span>{item.label}</span>
                </a>
              );
            })}
          </div>

          <div className="border-t border-border pt-3 space-y-3">
            {/* Language Switcher */}
            <div className="flex items-center justify-between px-1">
              <span className="flex items-center gap-2 text-xs font-medium text-secondary">
                <Globe className="h-4 w-4" />
                {t('common.language')}
              </span>
              <div className="inline-flex rounded-lg border border-border bg-surface-muted p-0.5">
                <button
                  type="button"
                  onClick={() => setLocale('en')}
                  className={cn(
                    'min-h-9 px-3 py-1 rounded-md text-xs font-semibold transition-colors cursor-pointer',
                    locale === 'en'
                      ? 'bg-surface text-primary shadow-xs'
                      : 'text-secondary hover:text-primary'
                  )}
                >
                  EN
                </button>
                <button
                  type="button"
                  onClick={() => setLocale('ru')}
                  className={cn(
                    'min-h-9 px-3 py-1 rounded-md text-xs font-semibold transition-colors cursor-pointer',
                    locale === 'ru'
                      ? 'bg-surface text-primary shadow-xs'
                      : 'text-secondary hover:text-primary'
                  )}
                >
                  RU
                </button>
              </div>
            </div>

            {/* Theme Switcher */}
            {onThemeChange && (
              <div className="flex items-center justify-between px-1">
                <span className="text-xs font-medium text-secondary">
                  {t('common.themeSelector')}
                </span>
                <div className="inline-flex rounded-lg border border-border bg-surface-muted p-0.5">
                  <button
                    type="button"
                    onClick={() => onThemeChange('light')}
                    className={cn(
                      'inline-flex min-h-9 items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
                      themeMode === 'light'
                        ? 'bg-surface text-primary shadow-xs'
                        : 'text-secondary hover:text-primary'
                    )}
                  >
                    <Sun className="h-3.5 w-3.5" />
                    <span>{t('common.lightMode')}</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => onThemeChange('dark')}
                    className={cn(
                      'inline-flex min-h-9 items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
                      themeMode === 'dark'
                        ? 'bg-surface text-primary shadow-xs'
                        : 'text-secondary hover:text-primary'
                    )}
                  >
                    <Moon className="h-3.5 w-3.5" />
                    <span>{t('common.darkMode')}</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => onThemeChange('system')}
                    className={cn(
                      'inline-flex min-h-9 items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
                      themeMode === 'system'
                        ? 'bg-surface text-primary shadow-xs'
                        : 'text-secondary hover:text-primary'
                    )}
                  >
                    <Monitor className="h-3.5 w-3.5" />
                    <span>{t('common.systemMode')}</span>
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </Drawer>
    </>
  );
};
