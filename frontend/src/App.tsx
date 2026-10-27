import React, { useState, useEffect, Suspense, useCallback } from 'react';
import { Home, MapPin, User } from 'lucide-react';
import {
  AccessDeniedCard,
  AppShell,
  BrandLogoIcon,
  Button,
  Badge,
  Spinner,
} from './components/ui';
import { TreeCanvas, TreePrivacyPreviewCard } from './components/tree';
import { EmptyTreeOnboardingCard } from './components/onboarding';
import { SettingsPage } from './pages/SettingsPage';
import { PlacesPage } from './pages/PlacesPage';
import { PersonProfilePage } from './pages/PersonProfilePage';
import { AuthPages } from './pages/AuthPages';
import {
  fetchOnboardingStatus,
  fetchSystemInfo,
  OnboardingStatusResponse,
} from './api/client';
import { useAuth } from './lib/auth-context';
import { useI18n } from './i18n';

const DevUiWorkbench = React.lazy(() => import('./pages/DevUiWorkbench'));

export interface AppProps {
  initialOnboardingStatus?: OnboardingStatusResponse | null;
}

export const App: React.FC<AppProps> = ({ initialOnboardingStatus = null }) => {
  const { t } = useI18n();
  const { user, setupStatus, isAuthenticated, isLoading, backendReachable } =
    useAuth();
  const [path, setPath] = useState<string>(() =>
    typeof window !== 'undefined' ? window.location.pathname : '/'
  );
  const [search, setSearch] = useState<string>(() =>
    typeof window !== 'undefined' ? window.location.search : ''
  );
  const [instanceTitle, setInstanceTitle] = useState<string | undefined>(
    undefined
  );
  const [onboardingStatus, setOnboardingStatus] =
    useState<OnboardingStatusResponse | null>(initialOnboardingStatus);

  useEffect(() => {
    const onPopState = () => {
      setPath(window.location.pathname);
      setSearch(window.location.search);
    };
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, []);

  useEffect(() => {
    let cancelled = false;
    fetchSystemInfo()
      .then((info) => {
        if (!cancelled && info.tree_title) {
          setInstanceTitle(info.tree_title);
          if (typeof document !== 'undefined') {
            document.title = info.tree_title;
          }
        }
      })
      .catch(() => {
        // Fallback gracefully to localized default when backend is offline
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const loadOnboardingStatus = useCallback(() => {
    if (initialOnboardingStatus) return;
    fetchOnboardingStatus()
      .then((status) => {
        setOnboardingStatus(status);
      })
      .catch(() => {});
  }, [initialOnboardingStatus]);

  useEffect(() => {
    if (initialOnboardingStatus) {
      setOnboardingStatus(initialOnboardingStatus);
      return;
    }
    loadOnboardingStatus();
  }, [initialOnboardingStatus, loadOnboardingStatus, isAuthenticated]);

  const navigate = (nextPath: string) => {
    const baseOrigin =
      typeof window !== 'undefined'
        ? window.location.origin
        : 'http://localhost';
    const parsed = new URL(nextPath, baseOrigin);
    if (typeof window !== 'undefined') {
      window.history.pushState({}, '', `${parsed.pathname}${parsed.search}`);
    }
    setPath(parsed.pathname);
    setSearch(parsed.search);
  };

  // 1. Dev UI Workbench route
  if (path.startsWith('/dev/ui')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'dev-ui', label: t('nav.devWorkbench') },
        ]}
      >
        <Suspense
          fallback={
            <div className="flex min-h-[50vh] items-center justify-center gap-2 text-sm text-secondary">
              <Spinner size="md" />
              <span>{t('common.loading')}</span>
            </div>
          }
        >
          <DevUiWorkbench />
        </Suspense>
      </AppShell>
    );
  }

  // 2. First-run uninitialized instance guard -> /setup (01 §2.1)
  if (
    !isLoading &&
    backendReachable &&
    setupStatus &&
    !setupStatus.is_initialized
  ) {
    return (
      <AuthPages
        currentPath="/setup"
        instanceTitle={instanceTitle || setupStatus.tree_title}
        onNavigate={navigate}
        onInstanceTitleChange={(nextTitle) => {
          setInstanceTitle(nextTitle);
          if (typeof document !== 'undefined') {
            document.title = nextTitle;
          }
        }}
      />
    );
  }

  // 3. Explicit Auth routes (/setup, /login, /register, /magic-link/verify)
  if (
    path.startsWith('/setup') ||
    path.startsWith('/login') ||
    path.startsWith('/register') ||
    path.startsWith('/magic-link/verify')
  ) {
    return (
      <AuthPages
        currentPath={path}
        search={search}
        instanceTitle={instanceTitle || setupStatus?.tree_title}
        onNavigate={navigate}
        onInstanceTitleChange={(nextTitle) => {
          setInstanceTitle(nextTitle);
          if (typeof document !== 'undefined') {
            document.title = nextTitle;
          }
        }}
      />
    );
  }

  // 4. Default Private Landing Guard (01-auth-pages.md §1, §4 DoD #1):
  // When instance is initialized, private (enable_public_access === false), and visitor is unauthenticated
  if (
    !isLoading &&
    backendReachable &&
    setupStatus &&
    setupStatus.is_initialized &&
    !setupStatus.enable_public_access &&
    !isAuthenticated
  ) {
    return (
      <AuthPages
        currentPath="/login"
        instanceTitle={instanceTitle || setupStatus.tree_title}
        onNavigate={navigate}
      />
    );
  }

  // 4b. Public Tree Mode Guard (02-rbac-access-control.md §3.2):
  // When enable_public_access === true and visitor is unauthenticated, only /tree and /persons/* are accessible;
  // auxiliary routes (/people, /media, /places, /settings, /history, /quality, /export/print) redirect to /login
  if (
    !isLoading &&
    backendReachable &&
    setupStatus &&
    setupStatus.is_initialized &&
    setupStatus.enable_public_access &&
    !isAuthenticated &&
    (path.startsWith('/people') ||
      path.startsWith('/media') ||
      path.startsWith('/places') ||
      path.startsWith('/settings') ||
      path.startsWith('/history') ||
      path.startsWith('/quality') ||
      path.startsWith('/export'))
  ) {
    return (
      <AuthPages
        currentPath="/login"
        instanceTitle={instanceTitle || setupStatus.tree_title}
        onNavigate={navigate}
      />
    );
  }

  // 4c. Admin-only route RBAC guard (/history, /quality, /people/merge, /export/print) (02 §2 & §7 DoD #6, #7, #8)
  if (
    path.startsWith('/history') ||
    path.startsWith('/quality') ||
    path.startsWith('/people/merge') ||
    path.startsWith('/export/print')
  ) {
    if (user && user.role !== 'admin') {
      return (
        <AppShell
          currentPath={path}
          onNavigate={navigate}
          instanceTitle={instanceTitle}
          breadcrumbs={[
            { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
            { id: 'restricted', label: t('rbac.adminOnlyBadge') },
          ]}
        >
          <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-12">
            <AccessDeniedCard
              currentRole={user.role}
              onOpenProfile={() => navigate('/settings/profile')}
              onBackToTree={() => navigate('/tree')}
            />
          </div>
        </AppShell>
      );
    }
  }

  // 5. Tree route (/tree) — renders interactive full-viewport TreeCanvas (02-tree-view-page.md, 04-tree-visualizations.md)
  if (path.startsWith('/tree')) {
    const focusParam = new URLSearchParams(search).get('focus');
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
      >
        <TreeCanvas
          focusPersonId={focusParam}
          onSignIn={() => navigate('/login')}
          onNavigate={navigate}
        />
      </AppShell>
    );
  }

  // 5a. Person Profile route (/persons/:id) — (03-person-profile-page.md)
  if (path.startsWith('/persons/')) {
    const rawPersonId = decodeURIComponent(
      path.slice('/persons/'.length).split('/')[0] || ''
    );
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'tree', label: t('nav.tree'), onClick: () => navigate('/tree') },
          { id: 'person', label: rawPersonId || t('profile.breadcrumbLabel') },
        ]}
      >
        <PersonProfilePage
          personId={rawPersonId}
          onNavigate={navigate}
          onSignIn={() => navigate('/login')}
        />
      </AppShell>
    );
  }

  // 5b. Places & Historical Jurisdictions route (/places)
  if (path.startsWith('/places')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'places', label: t('nav.places') },
        ]}
      >
        <PlacesPage />
      </AppShell>
    );
  }

  // 6. Settings & Administration routes (/settings/*)
  if (path.startsWith('/settings')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'settings', label: t('settings.pageTitle') },
        ]}
      >
        <SettingsPage
          currentPath={path}
          onNavigate={navigate}
          onInstanceTitleChange={(nextTitle) => {
            setInstanceTitle(nextTitle);
            if (typeof document !== 'undefined') {
              document.title = nextTitle;
            }
          }}
        />
      </AppShell>
    );
  }

  // 7. Home route (/) — Empty-tree onboarding or start_page preference (16-dashboard-home.md §2, §3, §4)
  const isTreeEmpty = Boolean(onboardingStatus?.is_empty);
  const effectiveStartPage =
    user?.start_page ?? onboardingStatus?.start_page ?? 'dashboard';
  const resolvedHomePersonId =
    onboardingStatus?.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

  // Unauthenticated visitors on a public instance or users who chose 'tree' as their start page
  const openTreeDirectly =
    !isTreeEmpty &&
    (effectiveStartPage === 'tree' ||
      (Boolean(setupStatus?.enable_public_access) && !isAuthenticated));

  return (
    <AppShell
      currentPath={path}
      onNavigate={navigate}
      instanceTitle={instanceTitle}
    >
      <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {isTreeEmpty ? (
          <EmptyTreeOnboardingCard
            canOnboard={onboardingStatus?.can_onboard}
            userRole={user?.role ?? (isAuthenticated ? 'viewer' : 'viewer')}
            onPersonCreated={() => {
              loadOnboardingStatus();
              navigate('/tree');
            }}
            onImportGedcom={() => navigate('/settings/gedcom')}
          />
        ) : openTreeDirectly ? (
          <TreeCanvas
            onSignIn={() => navigate('/login')}
            onNavigate={navigate}
          />
        ) : (
          <>
            {/* Calm Family Archive Overview Header */}
            <section className="rounded-3xl border border-border bg-surface p-6 sm:p-10 shadow-xs">
              <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6">
                <div className="max-w-2xl space-y-2.5">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="brand">{t('app.badge')}</Badge>
                    {resolvedHomePersonId && (
                      <Badge
                        variant="neutral"
                        icon={<Home className="h-3 w-3" />}
                      >
                        {t('tree.homePersonBadge')}: {resolvedHomePersonId}
                      </Badge>
                    )}
                  </div>

                  <h1 className="font-sans text-2xl sm:text-3xl font-semibold tracking-tight text-primary">
                    {user?.first_name
                      ? t('onboarding.dashboardGreeting', {
                          name: user.first_name,
                        })
                      : instanceTitle || t('app.title')}
                  </h1>

                  <p className="text-sm sm:text-base text-secondary leading-relaxed">
                    {t('app.subtitle')}. {t('app.heroDescription')}
                  </p>
                </div>

                <div className="flex flex-wrap items-center gap-2.5 shrink-0">
                  <Button
                    variant="primary"
                    size="md"
                    leftIcon={<BrandLogoIcon className="h-4 w-4" />}
                    onClick={() => navigate('/tree')}
                  >
                    {t('app.openWorkbench')}
                  </Button>
                  <Button
                    variant="secondary"
                    size="md"
                    leftIcon={<MapPin className="h-4 w-4" />}
                    onClick={() => navigate('/places')}
                  >
                    {t('nav.places')}
                  </Button>
                  <Button
                    variant="ghost"
                    size="md"
                    leftIcon={<User className="h-4 w-4" />}
                    onClick={() => navigate('/settings/profile')}
                  >
                    {t('nav.myProfile')}
                  </Button>
                </div>
              </div>
            </section>

            {/* Family Tree Records & Home Person Preview */}
            <TreePrivacyPreviewCard
              onSignIn={() => navigate('/login')}
              onNavigate={navigate}
            />
          </>
        )}
      </div>
    </AppShell>
  );
};

export default App;
