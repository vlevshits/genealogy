import React, { useState } from 'react';
import {
  ChevronDown,
  LogIn,
  Menu,
  Plus,
  Search,
  User,
} from 'lucide-react';
import { useI18n } from '../../i18n';
import { useTheme } from '../../lib/theme';
import { useAuth } from '../../lib/auth-context';
import { NavLink } from './nav-link';
import { DropdownMenu } from './dropdown-menu';
import { Kbd } from './kbd';
import { Button } from './button';
import { IconButton } from './icon-button';
import { BrandLogoIcon } from './brand-logo-icon';
import { LanguageToggleButton } from './language-toggle-button';
import { ThemeToggleButton } from './theme-toggle-button';
import { Drawer } from './drawer';
import { BottomNav } from './bottom-nav';
import { Breadcrumbs, BreadcrumbItem } from './breadcrumbs';
import { OmniboxDialog } from '../search';
import { PwaInstallPrompt, PwaOfflineBanner } from '../pwa';
import { buildLoginUrl } from '../../lib/navigation-utils';
import { cn } from '../../lib/utils';

export interface AppShellProps {
  currentPath?: string;
  onNavigate?: (path: string) => void;
  instanceTitle?: string;
  breadcrumbs?: BreadcrumbItem[];
  contextAction?: React.ReactNode;
  children: React.ReactNode;
}

export const AppShell: React.FC<AppShellProps> = ({
  currentPath = '/',
  onNavigate,
  instanceTitle,
  breadcrumbs,
  contextAction,
  children,
}) => {
  const { t } = useI18n();
  const { theme, setTheme } = useTheme();
  const { user, isAuthenticated, logout } = useAuth();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [omniboxOpen, setOmniboxOpen] = useState(false);
  const resolvedTitle = instanceTitle || t('app.instanceName');

  const navigate = (path: string) => {
    setMobileMenuOpen(false);
    if (typeof window !== 'undefined' && typeof window.scrollTo === 'function') {
      window.scrollTo(0, 0);
    }
    if (onNavigate) {
      onNavigate(path);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', path);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  const handleSignOut = async () => {
    await logout();
    navigate('/login');
  };

  const userInitials = user
    ? `${user.first_name?.[0] ?? ''}${user.last_name?.[0] ?? ''}`.toUpperCase() || 'U'
    : null;

  const isFullViewportRoute =
    currentPath.startsWith('/tree') || currentPath.startsWith('/export/print');

  return (
    <div
      className={cn(
        'flex flex-col bg-canvas text-primary transition-colors duration-200',
        isFullViewportRoute
          ? 'fixed inset-0 overflow-hidden'
          : 'min-h-screen pb-20 md:pb-0'
      )}
    >
      {/* Persistent Top Header */}
      <PwaOfflineBanner />
      <header className="sticky top-0 z-30 shrink-0 w-full border-b border-border bg-surface/90 backdrop-blur-md pt-[env(safe-area-inset-top,0px)]">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between gap-2 sm:gap-3 px-3 sm:px-6 lg:px-8">
          {/* Left: Mobile Hamburger + Brand + Primary Links */}
          <div className="flex items-center gap-2 lg:gap-3 min-w-0 flex-1">
            <div className="md:hidden shrink-0">
              <IconButton
                label={t('nav.openMenu')}
                onClick={() => setMobileMenuOpen(true)}
              >
                <Menu className="h-5 w-5" />
              </IconButton>
            </div>

            <a
              href="/"
              title={resolvedTitle}
              onClick={(e) => {
                e.preventDefault();
                navigate('/');
              }}
              className="flex items-center gap-2 sm:gap-2.5 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus min-w-0 flex-1 md:flex-initial"
            >
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-brand text-surface shadow-xs">
                <BrandLogoIcon className="h-5 w-5" />
              </div>
              <span className="font-sans text-sm sm:text-base lg:text-lg font-semibold tracking-tight text-primary whitespace-nowrap truncate min-w-0">
                {resolvedTitle}
              </span>
            </a>

            {/* Primary Nav Links */}
            <nav aria-label={t('nav.mainNav')} className="hidden md:flex items-center gap-0.5 lg:gap-1 ml-1 lg:ml-3 shrink-0">
              <NavLink
                href="/"
                active={currentPath === '/'}
                onNavigate={navigate}
                className="px-2 lg:px-3 text-xs lg:text-sm"
              >
                {t('nav.home')}
              </NavLink>
              <NavLink
                href="/tree"
                active={currentPath.startsWith('/tree')}
                onNavigate={navigate}
                className="px-2 lg:px-3 text-xs lg:text-sm"
              >
                {t('nav.tree')}
              </NavLink>
              {isAuthenticated && (
                <>
                  <NavLink
                    href="/people"
                    active={currentPath.startsWith('/people')}
                    onNavigate={navigate}
                    className="px-2 lg:px-3 text-xs lg:text-sm"
                  >
                    {t('nav.people')}
                  </NavLink>
                  <NavLink
                    href="/media"
                    active={currentPath.startsWith('/media')}
                    onNavigate={navigate}
                    className="px-2 lg:px-3 text-xs lg:text-sm"
                  >
                    {t('nav.media')}
                  </NavLink>

                  <DropdownMenu
                    align="start"
                    ariaLabel={t('nav.more')}
                    trigger={
                      <span className="inline-flex min-h-11 items-center gap-1 rounded-lg px-2 lg:px-3 py-1.5 text-xs lg:text-sm font-medium text-secondary hover:bg-surface-muted hover:text-primary">
                        <span>{t('nav.more')}</span>
                        <ChevronDown className="h-3.5 w-3.5" />
                      </span>
                    }
                    items={[
                      { id: 'places', label: t('nav.places'), onSelect: () => navigate('/places') },
                      { id: 'print', label: t('nav.print'), onSelect: () => navigate('/export/print') },
                      ...(user?.role === 'editor' || user?.role === 'admin'
                        ? [
                            { id: 'story-intake', label: t('nav.storyIntake'), onSelect: () => navigate('/stories/intake') },
                          ]
                        : []),
                      ...(user?.role === 'admin'
                        ? [
                            {
                              id: 'history',
                              label: t('nav.history'),
                              separatorBefore: true,
                              onSelect: () => navigate('/history'),
                            },
                            { id: 'quality', label: t('nav.quality'), onSelect: () => navigate('/quality') },
                            { id: 'gedcom', label: t('nav.gedcom'), onSelect: () => navigate('/settings/gedcom') },
                          ]
                        : []),
                    ]}
                  />
                </>
              )}
            </nav>
          </div>

          {/* Right: Omnibox Trigger, +Add, Locale, Theme, User Menu */}
          <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
            {isAuthenticated && (
              <Button
                variant="secondary"
                size="md"
                aria-label={t('nav.searchTrigger')}
                onClick={() => setOmniboxOpen(true)}
                leftIcon={<Search className="h-3.5 w-3.5" />}
                rightIcon={<Kbd className="hidden sm:inline-flex">⌘K</Kbd>}
                className="rounded-xl bg-surface-muted/70 px-2.5 sm:px-3 py-1.5 text-xs text-secondary hover:bg-surface-muted hover:text-primary"
              >
                <span className="hidden xl:inline">{t('nav.searchTrigger')}</span>
              </Button>
            )}

            {isAuthenticated &&
              user &&
              (user.role === 'editor' || user.role === 'admin') && (
                <DropdownMenu
                  ariaLabel={t('nav.add')}
                  trigger={
                    <span className="inline-flex min-h-11 items-center gap-1 rounded-lg bg-brand px-2.5 sm:px-3 py-1.5 text-xs sm:text-sm font-medium text-surface hover:opacity-95 shadow-xs">
                      <Plus className="h-4 w-4 shrink-0" />
                      <span className="hidden lg:inline">{t('nav.add')}</span>
                    </span>
                  }
                  items={[
                    { id: 'add-person', label: t('nav.addPerson'), onSelect: () => navigate('/people') },
                    { id: 'upload-photos', label: t('nav.uploadPhotos'), onSelect: () => navigate('/media') },
                    { id: 'story-intake', label: t('nav.storyIntake'), onSelect: () => navigate('/stories/intake') },
                  ]}
                />
              )}

            {/* Reusable Language & Theme Toggle Buttons (hidden on < sm; available inside mobile Drawer) */}
            <div className="hidden sm:flex items-center gap-1.5">
              <LanguageToggleButton />
              <ThemeToggleButton />
            </div>

            {/* User Profile Menu or Sign-In Button in Public Mode */}
            {!isAuthenticated ? (
              <Button
                variant="primary"
                size="md"
                leftIcon={<LogIn className="h-4 w-4" />}
                className="hidden md:inline-flex"
                onClick={() =>
                  navigate(
                    buildLoginUrl(
                      currentPath,
                      typeof window !== 'undefined' ? window.location.search : undefined
                    )
                  )
                }
              >
                {t('nav.signIn')}
              </Button>
            ) : (
              <DropdownMenu
                ariaLabel={t('nav.userMenu')}
                trigger={
                  <span
                    aria-label={t('nav.userMenu')}
                    title={user?.display_name ?? t('nav.userMenu')}
                    className="inline-flex h-11 w-11 items-center justify-center rounded-full border border-border bg-brand/15 text-xs font-semibold text-brand"
                  >
                    {userInitials ?? <User className="h-4 w-4" />}
                  </span>
                }
                items={[
                  {
                    id: 'profile',
                    label: user ? `${user.display_name} (${t('nav.myProfile')})` : t('nav.myProfile'),
                    onSelect: () => navigate('/settings/profile'),
                  },
                  { id: 'passkeys', label: t('nav.securityPasskeys'), onSelect: () => navigate('/settings/security') },
                  ...(user?.role === 'admin'
                    ? [{ id: 'admin', label: t('nav.adminSettings'), onSelect: () => navigate('/settings') }]
                    : []),
                  {
                    id: 'logout',
                    label: t('nav.signOut'),
                    danger: true,
                    separatorBefore: true,
                    onSelect: () => {
                      void handleSignOut();
                    },
                  },
                ]}
              />
            )}
          </div>
        </div>

        {/* Context Sub-bar */}
        {(breadcrumbs || contextAction) && (
          <div className="border-t border-border bg-surface/60">
            <div className="mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-2 sm:px-6 lg:px-8">
              {breadcrumbs ? <Breadcrumbs items={breadcrumbs} /> : <div />}
              {contextAction && <div>{contextAction}</div>}
            </div>
          </div>
        )}
      </header>

      {/* Mobile Hamburger Drawer (< 768px) */}
      <Drawer
        open={mobileMenuOpen}
        onOpenChange={setMobileMenuOpen}
        side="right"
        title={resolvedTitle}
      >
        <div className="flex flex-col gap-1">
          <NavLink href="/" active={currentPath === '/'} onNavigate={navigate}>
            {t('nav.home')}
          </NavLink>
          <NavLink href="/tree" active={currentPath.startsWith('/tree')} onNavigate={navigate}>
            {t('nav.tree')}
          </NavLink>
          {isAuthenticated ? (
            <>
              <NavLink href="/people" active={currentPath.startsWith('/people')} onNavigate={navigate}>
                {t('nav.people')}
              </NavLink>
              <NavLink href="/media" active={currentPath.startsWith('/media')} onNavigate={navigate}>
                {t('nav.media')}
              </NavLink>
              <NavLink href="/places" active={currentPath.startsWith('/places')} onNavigate={navigate}>
                {t('nav.places')}
              </NavLink>
              {(user?.role === 'editor' || user?.role === 'admin') && (
                <NavLink href="/stories/intake" active={currentPath.startsWith('/stories/intake')} onNavigate={navigate}>
                  {t('nav.storyIntake')}
                </NavLink>
              )}
              {user?.role === 'admin' && (
                <>
                  <NavLink href="/quality" active={currentPath.startsWith('/quality')} onNavigate={navigate}>
                    {t('nav.quality')}
                  </NavLink>
                  <NavLink href="/settings" active={currentPath.startsWith('/settings')} onNavigate={navigate}>
                    {t('nav.adminSettings')}
                  </NavLink>
                </>
              )}
            </>
          ) : (
            <div className="pt-2 mt-2 border-t border-border">
              <Button
                variant="primary"
                size="md"
                leftIcon={<LogIn className="h-4 w-4" />}
                onClick={() =>
                  navigate(
                    buildLoginUrl(
                      currentPath,
                      typeof window !== 'undefined' ? window.location.search : undefined
                    )
                  )
                }
                className="w-full justify-center"
              >
                {t('nav.signIn')}
              </Button>
            </div>
          )}

          {/* Mobile Drawer Language & Theme Controls */}
          <div className="mt-4 pt-3 border-t border-border flex items-center justify-between gap-2">
            <span className="text-xs font-medium text-secondary">{t('settings.tabs.appearance')}</span>
            <div className="flex items-center gap-1.5">
              <LanguageToggleButton />
              <ThemeToggleButton />
            </div>
          </div>
        </div>
      </Drawer>

      {/* Global Omnibox (Cmd+K) */}
      {isAuthenticated && (
        <OmniboxDialog
          open={omniboxOpen}
          onOpenChange={setOmniboxOpen}
          onNavigate={navigate}
        />
      )}

      {/* Main Content Viewport */}
      <PwaInstallPrompt instanceTitle={resolvedTitle} />
      <main
        className={cn(
          'flex-1 w-full',
          isFullViewportRoute && 'min-h-0 flex flex-col overflow-hidden'
        )}
      >
        {children}
      </main>

      {/* Footer (hidden on full-bleed /tree canvas and /export/print studio) */}
      {!currentPath.startsWith('/tree') && !currentPath.startsWith('/export/print') && (
        <footer className="mt-auto border-t border-border bg-surface py-5 text-xs text-secondary">
          <div className="mx-auto flex max-w-7xl flex-col sm:flex-row items-center justify-between gap-2 px-4 sm:px-6 lg:px-8">
            <span>{t('app.footerCopyright')}</span>
            <span>{t('app.footerRight')}</span>
          </div>
        </footer>
      )}

      {/* Mobile Bottom Navigation (< 768px) */}
      <BottomNav
        currentPath={currentPath}
        onNavigate={navigate}
        themeMode={theme}
        onThemeChange={setTheme}
      />
    </div>
  );
};
