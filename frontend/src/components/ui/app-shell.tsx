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
import { CommandPalette, CommandPaletteItem } from './command-palette';
import { Breadcrumbs, BreadcrumbItem } from './breadcrumbs';
import { SAMPLE_PEOPLE, SAMPLE_PLACES } from '../../lib/mock-data';

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

  const omniboxItems: CommandPaletteItem[] = [
    ...SAMPLE_PEOPLE.filter((p) => !p.is_redacted).map((p) => ({
      id: p.id,
      group: 'people' as const,
      title: `${p.surname ?? ''} ${p.given_name ?? ''} ${p.patronymic ?? ''}`.trim(),
      subtitle: `${p.gid} · ${p.birth_place ?? ''}`,
      onSelect: () => navigate(`/persons/${p.gid}`),
    })),
    ...SAMPLE_PLACES.map((pl) => ({
      id: pl.id,
      group: 'places' as const,
      title: pl.name,
      subtitle: pl.hierarchy_names?.join(', '),
      onSelect: () => navigate(`/places/${pl.gid}`),
    })),
    {
      id: 'act-tree',
      group: 'actions',
      title: t('nav.tree'),
      onSelect: () => navigate('/tree'),
    },
    {
      id: 'act-people',
      group: 'actions',
      title: t('nav.people'),
      onSelect: () => navigate('/people'),
    },
  ];

  const userInitials = user
    ? `${user.first_name?.[0] ?? ''}${user.last_name?.[0] ?? ''}`.toUpperCase() || 'U'
    : null;

  return (
    <div className="min-h-screen flex flex-col bg-canvas text-primary pb-20 md:pb-0 transition-colors duration-200">
      {/* Persistent Top Header */}
      <header className="sticky top-0 z-30 w-full border-b border-border bg-surface/90 backdrop-blur-md pt-[env(safe-area-inset-top,0px)]">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between gap-3 px-4 sm:px-6 lg:px-8">
          {/* Left: Mobile Hamburger + Brand + Primary Links */}
          <div className="flex items-center gap-3">
            <div className="md:hidden">
              <IconButton
                label={t('nav.openMenu')}
                onClick={() => setMobileMenuOpen(true)}
              >
                <Menu className="h-5 w-5" />
              </IconButton>
            </div>

            <a
              href="/"
              onClick={(e) => {
                e.preventDefault();
                navigate('/');
              }}
              className="flex items-center gap-2.5 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
            >
              <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-brand text-surface shadow-xs">
                <BrandLogoIcon className="h-5 w-5" />
              </div>
              <span className="font-sans text-base sm:text-lg font-semibold tracking-tight text-primary">
                {resolvedTitle}
              </span>
            </a>

            {/* Primary 4 Nav Links + More */}
            <nav aria-label="Main navigation" className="hidden md:flex items-center gap-1 ml-3">
              <NavLink
                href="/"
                active={currentPath === '/'}
                onNavigate={navigate}
              >
                {t('nav.home')}
              </NavLink>
              <NavLink
                href="/tree"
                active={currentPath.startsWith('/tree')}
                onNavigate={navigate}
              >
                {t('nav.tree')}
              </NavLink>
              <NavLink
                href="/people"
                active={currentPath.startsWith('/people')}
                onNavigate={navigate}
              >
                {t('nav.people')}
              </NavLink>
              <NavLink
                href="/media"
                active={currentPath.startsWith('/media')}
                onNavigate={navigate}
              >
                {t('nav.media')}
              </NavLink>

              <DropdownMenu
                align="start"
                ariaLabel={t('nav.more')}
                trigger={
                  <span className="inline-flex min-h-11 items-center gap-1 rounded-lg px-3 py-1.5 text-sm font-medium text-secondary hover:bg-surface-muted hover:text-primary">
                    <span>{t('nav.more')}</span>
                    <ChevronDown className="h-3.5 w-3.5" />
                  </span>
                }
                items={[
                  { id: 'places', label: t('nav.places'), onSelect: () => navigate('/places') },
                  { id: 'print', label: t('nav.print'), onSelect: () => navigate('/export/print') },
                  {
                    id: 'history',
                    label: t('nav.history'),
                    separatorBefore: true,
                    onSelect: () => navigate('/history'),
                  },
                  { id: 'quality', label: t('nav.quality'), onSelect: () => navigate('/quality') },
                  { id: 'gedcom', label: t('nav.gedcom'), onSelect: () => navigate('/settings/gedcom') },
                ]}
              />
            </nav>
          </div>

          {/* Right: Omnibox Trigger, +Add, Locale, Theme, User Menu */}
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              size="md"
              aria-label={t('nav.searchTrigger')}
              onClick={() => setOmniboxOpen(true)}
              leftIcon={<Search className="h-3.5 w-3.5" />}
              rightIcon={<Kbd className="hidden sm:inline-flex">⌘K</Kbd>}
              className="rounded-xl bg-surface-muted/70 px-3 py-1.5 text-xs text-secondary hover:bg-surface-muted hover:text-primary"
            >
              <span className="hidden lg:inline">{t('nav.searchTrigger')}</span>
            </Button>

            {(!user || user.role !== 'viewer') && (
              <DropdownMenu
                ariaLabel={t('nav.add')}
                trigger={
                  <span className="inline-flex min-h-11 items-center gap-1 rounded-lg bg-brand px-3 py-1.5 text-xs sm:text-sm font-medium text-surface hover:opacity-95 shadow-xs">
                    <Plus className="h-4 w-4" />
                    <span className="hidden sm:inline">{t('nav.add')}</span>
                  </span>
                }
                items={[
                  { id: 'add-person', label: t('nav.addPerson'), onSelect: () => navigate('/people') },
                  { id: 'upload-photos', label: t('nav.uploadPhotos'), onSelect: () => navigate('/media') },
                ]}
              />
            )}

            {/* Reusable Language & Theme Toggle Buttons */}
            <LanguageToggleButton />
            <ThemeToggleButton />

            {/* User Profile Menu or Sign-In Button in Public Mode */}
            {!isAuthenticated && currentPath !== '/login' && user === null ? (
              <DropdownMenu
                ariaLabel={t('nav.userMenu')}
                trigger={
                  <span
                    aria-label={t('nav.userMenu')}
                    className="inline-flex h-11 w-11 items-center justify-center rounded-full border border-border bg-brand/15 text-xs font-semibold text-brand"
                  >
                    <User className="h-4 w-4" />
                  </span>
                }
                items={[
                  { id: 'profile', label: t('nav.myProfile'), onSelect: () => navigate('/settings/profile') },
                  { id: 'passkeys', label: t('nav.securityPasskeys'), onSelect: () => navigate('/settings/security') },
                  { id: 'admin', label: t('nav.adminSettings'), onSelect: () => navigate('/settings') },
                  {
                    id: 'signin',
                    label: t('nav.signIn'),
                    icon: <LogIn className="h-3.5 w-3.5" />,
                    separatorBefore: true,
                    onSelect: () => navigate('/login'),
                  },
                ]}
              />
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
                  ...(!user || user.role === 'admin'
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
          <NavLink href="/people" active={currentPath.startsWith('/people')} onNavigate={navigate}>
            {t('nav.people')}
          </NavLink>
          <NavLink href="/media" active={currentPath.startsWith('/media')} onNavigate={navigate}>
            {t('nav.media')}
          </NavLink>
          <NavLink href="/places" active={currentPath.startsWith('/places')} onNavigate={navigate}>
            {t('nav.places')}
          </NavLink>
          <NavLink href="/settings" active={currentPath.startsWith('/settings')} onNavigate={navigate}>
            {t('nav.adminSettings')}
          </NavLink>
        </div>
      </Drawer>

      {/* Global Command Palette (Cmd+K) */}
      <CommandPalette
        open={omniboxOpen}
        onOpenChange={setOmniboxOpen}
        items={omniboxItems}
      />

      {/* Main Content Viewport */}
      <main className="flex-1 w-full">{children}</main>

      {/* Footer */}
      <footer className="mt-auto border-t border-border bg-surface py-5 text-xs text-secondary">
        <div className="mx-auto flex max-w-7xl flex-col sm:flex-row items-center justify-between gap-2 px-4 sm:px-6 lg:px-8">
          <span>{t('app.footerCopyright')}</span>
          <span>{t('app.footerRight')}</span>
        </div>
      </footer>

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
