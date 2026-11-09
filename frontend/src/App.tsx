import React, { useState, useEffect, Suspense, useCallback, useRef } from 'react';
import {
  AccessDeniedCard,
  AppShell,
  BrandLogoIcon,
  Spinner,
} from './components/ui';
import { TreeCanvas } from './components/tree';
import { EmptyTreeOnboardingCard } from './components/onboarding';
import { SettingsPage } from './pages/SettingsPage';
import { PlacesPage } from './pages/PlacesPage';
import { PersonProfilePage } from './pages/PersonProfilePage';
import { PeoplePage } from './pages/PeoplePage';
import { SurnamesPage } from './pages/SurnamesPage';
import { MediaPage } from './pages/MediaPage';
import { HistoryPage } from './pages/HistoryPage';
import { DataQualityPage } from './pages/DataQualityPage';
import { PersonMergePage } from './pages/PersonMergePage';
import { PrintExportPage } from './pages/PrintExportPage';
import { AiStoryIntakePage } from './pages/AiStoryIntakePage';
import { DashboardPage } from './pages/DashboardPage';
import { AuthPages } from './pages/AuthPages';
import {
  fetchOnboardingStatus,
  fetchSystemInfo,
  OnboardingStatusResponse,
  TREE_DATA_UPDATED_EVENT,
} from './api/client';
import { useAuth } from './lib/auth-context';
import { buildLoginUrl } from './lib/navigation-utils';
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
  const [activePersonName, setActivePersonName] = useState<string>('');
  const [onboardingStatus, setOnboardingStatus] =
    useState<OnboardingStatusResponse | null>(initialOnboardingStatus);
  const isTreeEmpty = Boolean(onboardingStatus?.is_empty);

  const initialRedirectAttemptedRef = useRef(false);

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
    if (!isAuthenticated && !setupStatus?.enable_public_access) return;
    fetchOnboardingStatus()
      .then((status) => {
        setOnboardingStatus(status);
      })
      .catch(() => {
        setOnboardingStatus({
          is_empty: false,
          total_individuals: 1,
          can_onboard: false,
          resolved_home_person_id: null,
          default_home_person_id: null,
          user_home_person_id: null,
          user_person_id: null,
          start_page: 'dashboard',
        });
      });
  }, [
    initialOnboardingStatus,
    isAuthenticated,
    setupStatus?.enable_public_access,
  ]);

  useEffect(() => {
    if (initialOnboardingStatus) {
      setOnboardingStatus(initialOnboardingStatus);
      return;
    }
    loadOnboardingStatus();
  }, [initialOnboardingStatus, loadOnboardingStatus, isAuthenticated]);

  useEffect(() => {
    if (path === '/') {
      loadOnboardingStatus();
    }
  }, [path, loadOnboardingStatus]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const handleTreeUpdated = () => {
      loadOnboardingStatus();
    };
    window.addEventListener(TREE_DATA_UPDATED_EVENT, handleTreeUpdated);
    return () => {
      window.removeEventListener(TREE_DATA_UPDATED_EVENT, handleTreeUpdated);
    };
  }, [loadOnboardingStatus]);

  const navigate = useCallback((nextPath: string) => {
    initialRedirectAttemptedRef.current = true;
    if (typeof window !== 'undefined') {
      try {
        sessionStorage.setItem('genealogy_start_page_redirected', 'true');
      } catch {
        // Ignore storage access error
      }
    }
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
  }, []);

  // Default Start Page redirect on cold start / initial application load (16-dashboard-home.md)
  useEffect(() => {
    if (isLoading) return;

    let alreadyRedirected = false;
    if (typeof window !== 'undefined') {
      try {
        alreadyRedirected =
          sessionStorage.getItem('genealogy_start_page_redirected') === 'true';
      } catch {
        // Ignore storage access error
      }
    }

    if (initialRedirectAttemptedRef.current || alreadyRedirected) {
      return;
    }

    // Wait until onboardingStatus has loaded if authenticated and not yet provided
    if (isAuthenticated && !initialOnboardingStatus && onboardingStatus === null) {
      return;
    }

    // Only apply start_page redirect if the user landed on '/'
    if (path === '/') {
      if (isAuthenticated && !isTreeEmpty && user?.start_page === 'tree') {
        initialRedirectAttemptedRef.current = true;
        if (typeof window !== 'undefined') {
          try {
            sessionStorage.setItem('genealogy_start_page_redirected', 'true');
          } catch {
            // Ignore storage access error
          }
        }
        navigate('/tree');
        return;
      }
    }

    // Mark attempted so subsequent in-app navigations to '/' stay on '/'
    initialRedirectAttemptedRef.current = true;
    if (typeof window !== 'undefined') {
      try {
        sessionStorage.setItem('genealogy_start_page_redirected', 'true');
      } catch {
        // Ignore storage access error
      }
    }
  }, [
    isLoading,
    isAuthenticated,
    initialOnboardingStatus,
    onboardingStatus,
    isTreeEmpty,
    user?.start_page,
    path,
    navigate,
  ]);

  // Determine whether this route is restricted for unauthenticated visitors in public mode (02 §3.2)
  const isRestrictedUnauthenticatedRoute =
    !isAuthenticated &&
    Boolean(setupStatus?.enable_public_access) &&
    (path.startsWith('/people') ||
      path.startsWith('/media') ||
      path.startsWith('/places') ||
      path.startsWith('/settings') ||
      path.startsWith('/history') ||
      path.startsWith('/quality') ||
      path.startsWith('/export') ||
      path === '/stories/intake' ||
      path.startsWith('/stories/') ||
      (path !== '/' &&
        !path.startsWith('/tree') &&
        !path.startsWith('/persons/') &&
        !path.startsWith('/login') &&
        !path.startsWith('/register') &&
        !path.startsWith('/setup') &&
        !path.startsWith('/magic-link/verify') &&
        !path.startsWith('/dev/ui')));

  // In public tree mode: redirect restricted auxiliary routes to /login with returnUrl
  useEffect(() => {
    if (
      !isLoading &&
      !isAuthenticated &&
      setupStatus?.enable_public_access &&
      isRestrictedUnauthenticatedRoute &&
      typeof window !== 'undefined' &&
      !window.location.pathname.startsWith('/login')
    ) {
      const returnUrl = `${path}${search}`;
      window.history.replaceState(
        {},
        '',
        `/login?returnUrl=${encodeURIComponent(returnUrl)}`
      );
    }
  }, [
    isLoading,
    isAuthenticated,
    setupStatus?.enable_public_access,
    isRestrictedUnauthenticatedRoute,
    path,
    search,
  ]);

  // In default private archive mode: update address bar cleanly to /login when unauthenticated
  useEffect(() => {
    if (
      !isLoading &&
      backendReachable &&
      setupStatus?.is_initialized &&
      !setupStatus.enable_public_access &&
      !isAuthenticated &&
      typeof window !== 'undefined' &&
      !window.location.pathname.startsWith('/login') &&
      !window.location.pathname.startsWith('/register') &&
      !window.location.pathname.startsWith('/setup') &&
      !window.location.pathname.startsWith('/magic-link/verify') &&
      !window.location.pathname.startsWith('/dev/ui')
    ) {
      const returnUrl = path !== '/' ? `${path}${search}` : '';
      const targetUrl = returnUrl
        ? `/login?returnUrl=${encodeURIComponent(returnUrl)}`
        : '/login';
      if (window.location.pathname + window.location.search !== targetUrl) {
        window.history.replaceState({}, '', targetUrl);
      }
    }
  }, [
    isLoading,
    backendReachable,
    setupStatus?.is_initialized,
    setupStatus?.enable_public_access,
    isAuthenticated,
    path,
    search,
  ]);

  // Render loading state while authenticating or fetching instance setup status
  if (isLoading && !initialOnboardingStatus) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center bg-surface gap-3 text-secondary">
        <BrandLogoIcon className="h-8 w-8 text-brand animate-pulse" />
        <Spinner size="lg" />
        <span className="text-sm font-medium">{t('common.loading')}</span>
      </div>
    );
  }

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
              <BrandLogoIcon className="h-4 w-4 text-brand" />
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
  if (backendReachable && setupStatus && !setupStatus.is_initialized) {
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
    backendReachable &&
    setupStatus &&
    setupStatus.is_initialized &&
    !setupStatus.enable_public_access &&
    !isAuthenticated
  ) {
    const returnUrl = path !== '/' ? `${path}${search}` : '';
    const loginSearch = returnUrl
      ? `?returnUrl=${encodeURIComponent(returnUrl)}`
      : '';
    return (
      <AuthPages
        currentPath="/login"
        search={loginSearch}
        instanceTitle={instanceTitle || setupStatus.tree_title}
        onNavigate={navigate}
      />
    );
  }

  // 4b. Public Tree Mode Guard (02-rbac-access-control.md §3.2):
  // When visitor is unauthenticated, permitted surface is ONLY: /, /tree, /persons/:id, /login, /register (and /setup, /magic-link/verify).
  // Auxiliary routes (/people*, /media*, /places*, /settings*, /history*, /quality*, /people/merge*, /export*) redirect to /login with returnUrl.
  if (!isAuthenticated && isRestrictedUnauthenticatedRoute) {
    const returnUrl = `${path}${search}`;
    return (
      <AuthPages
        currentPath="/login"
        search={`?returnUrl=${encodeURIComponent(returnUrl)}`}
        instanceTitle={instanceTitle || setupStatus?.tree_title}
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

  // 4d. History route (/history) — Admin-only global change history & undo/restore feed (14-change-history-concurrency.md §5)
  if (path.startsWith('/history')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'history', label: t('nav.history') },
        ]}
      >
        <HistoryPage onNavigate={navigate} />
      </AppShell>
    );
  }

  // 4e. Data Quality & Duplicate Detection route (/quality) — Admin-only (15-data-quality-duplicates.md §4)
  if (path.startsWith('/quality')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'quality', label: t('nav.quality') },
        ]}
      >
        <DataQualityPage onNavigate={navigate} />
      </AppShell>
    );
  }

  // 4f. Person Merge Wizard route (/people/merge) — Admin-only (15-data-quality-duplicates.md §5)
  if (path.startsWith('/people/merge')) {
    const mergeParams = new URLSearchParams(search);
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          {
            id: 'quality',
            label: t('nav.quality'),
            onClick: () => navigate('/quality?tab=duplicates'),
          },
          { id: 'merge', label: t('merge.pageTitle') },
        ]}
      >
        <PersonMergePage
          targetId={mergeParams.get('target') ?? undefined}
          sourceId={mergeParams.get('source') ?? undefined}
          onNavigate={navigate}
        />
      </AppShell>
    );
  }

  // 4g. Printable PDF Studio route (/export/print) — Admin-only (05-printable-pdf-page.md)
  if (path.startsWith('/export/print')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
      >
        <PrintExportPage onNavigate={navigate} />
      </AppShell>
    );
  }

  // 4h. AI Family Story Intake Studio route (/stories/intake) — Editor & Admin (10-ai-story-intake-page.md)
  if (path.startsWith('/stories/intake') || path.startsWith('/stories')) {
    if (user && user.role === 'viewer') {
      return (
        <AppShell
          currentPath={path}
          onNavigate={navigate}
          instanceTitle={instanceTitle}
          breadcrumbs={[
            { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
            { id: 'stories', label: t('nav.storyIntake') },
          ]}
        >
          <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-12">
            <AccessDeniedCard
              currentRole="viewer"
              onOpenProfile={() => navigate('/settings/profile')}
              onBackToTree={() => navigate('/tree')}
            />
          </div>
        </AppShell>
      );
    }

    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'stories', label: t('nav.storyIntake') },
        ]}
      >
        <AiStoryIntakePage onNavigate={navigate} />
      </AppShell>
    );
  }

  // 5. Tree route (/tree) — renders interactive full-viewport TreeCanvas (02-tree-view-page.md, 04-tree-visualizations.md)
  if (path.startsWith('/tree')) {
    const searchParams = new URLSearchParams(search);
    const focusParam = searchParams.get('focus') || searchParams.get('focal');
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
      >
        <TreeCanvas
          focusPersonId={focusParam}
          onSignIn={() => navigate(buildLoginUrl(path, search))}
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
          {
            id: 'person',
            label: activePersonName || t('profile.breadcrumbLabel'),
          },
        ]}
      >
        <PersonProfilePage
          personId={rawPersonId}
          onNavigate={navigate}
          onSignIn={() => navigate(buildLoginUrl(path, search))}
          onPersonLoaded={setActivePersonName}
        />
      </AppShell>
    );
  }

  // 5b. Surnames Index route (/people/surnames) — (13-search-people-directory.md §4.3)
  if (path.startsWith('/people/surnames')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          {
            id: 'people',
            label: t('nav.people'),
            onClick: () => navigate('/people'),
          },
          { id: 'surnames', label: t('surnames.title') },
        ]}
      >
        <SurnamesPage onNavigate={navigate} />
      </AppShell>
    );
  }

  // 5c. People Directory route (/people) — (13-search-people-directory.md §4)
  if (path === '/people' || (path.startsWith('/people') && !path.startsWith('/people/merge'))) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'people', label: t('nav.people') },
        ]}
      >
        <PeoplePage initialSearch={search} onNavigate={navigate} />
      </AppShell>
    );
  }

  // 5c-2. Media Archive & Face Tagging Studio route (/media) — (04-media-gallery-page.md, 06-media-face-tagging.md)
  if (path.startsWith('/media')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        instanceTitle={instanceTitle}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'media', label: t('nav.media') },
        ]}
      >
        <MediaPage initialSearch={search} onNavigate={navigate} />
      </AppShell>
    );
  }

  // 5d. Places & Historical Jurisdictions route (/places)
  if (path.startsWith('/places')) {
    const viewParam = new URLSearchParams(search).get('view');
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
        <PlacesPage
          initialView={viewParam === 'map' ? 'map' : 'list'}
          onNavigate={navigate}
        />
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

  // 7. Home route (/) — Empty-tree onboarding or Dashboard (16-dashboard-home.md §2, §3, §4)
  const resolvedHomePersonId =
    onboardingStatus?.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

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
        ) : (
          <DashboardPage
            instanceTitle={instanceTitle}
            resolvedHomePersonId={resolvedHomePersonId}
            onNavigate={navigate}
            onSignIn={() => navigate(buildLoginUrl(path, search))}
          />
        )}
      </div>
    </AppShell>
  );
};

export default App;
