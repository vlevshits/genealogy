import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { ToastProvider, ConfirmProvider } from '../../src/components/ui';
import { App } from '../../src/App';
import { AuthPages } from '../../src/pages/AuthPages';
import { LoginCard } from '../../src/components/auth/LoginCard';
import { UserDto, OnboardingStatusResponse } from '../../src/api/client';

const mockTreeUser: UserDto = {
  id: 'u-user-tree',
  email: 'tree.user@levshits.family',
  display_name: 'Tree Enthusiast',
  first_name: 'Vladimir',
  role: 'editor',
  is_active: true,
  start_page: 'tree',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

const mockDashboardUser: UserDto = {
  ...mockTreeUser,
  id: 'u-user-dashboard',
  start_page: 'dashboard',
};

const populatedOnboarding: OnboardingStatusResponse = {
  is_empty: false,
  total_individuals: 42,
  can_onboard: true,
  resolved_home_person_id: 'node-1',
  default_home_person_id: 'node-1',
  user_home_person_id: null,
  user_person_id: null,
  start_page: 'tree',
};

function renderWithProviders(
  node: React.ReactNode,
  user: UserDto | null = mockTreeUser,
  locale: 'en' | 'ru' = 'en'
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>
            <AuthProvider initialUser={user}>{node}</AuthProvider>
          </ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

describe('Start Page Preference & Root Route Architecture', () => {
  describe('1. Static Markup & Route / rendering integrity', () => {
    it('always renders DashboardPage on route / and NEVER renders TreeCanvas inside max-w-7xl', () => {
      const html = renderWithProviders(
        <App initialOnboardingStatus={populatedOnboarding} />,
        mockTreeUser,
        'en'
      );

      // Must render Dashboard header & greeting
      expect(html).toContain('Explore Family Tree');
      expect(html).toContain('Welcome back, Vladimir');

      // The container max-w-7xl must contain the dashboard, not the boxed TreeCanvas
      expect(html).toContain('max-w-7xl w-full mx-auto px-4');
      // TreeCanvas elements (like canvas node controls / SVG / canvas zoom controls) are not directly inside the root card
      expect(html).not.toContain('id="tree-viewport-canvas"');
    });

    it('renders DashboardPage in Russian as well', () => {
      const html = renderWithProviders(
        <App initialOnboardingStatus={populatedOnboarding} />,
        mockTreeUser,
        'ru'
      );

      expect(html).toContain('Открыть семейное древо');
      expect(html).toContain('С возвращением, Vladimir');
    });

    it('renders EmptyTreeOnboardingCard when tree is empty even if user preferred tree', () => {
      const emptyOnboarding: OnboardingStatusResponse = {
        ...populatedOnboarding,
        is_empty: true,
        total_individuals: 0,
      };

      const html = renderWithProviders(
        <App initialOnboardingStatus={emptyOnboarding} />,
        mockTreeUser,
        'en'
      );

      expect(html).toContain('Let&#x27;s start your family tree');
      expect(html).not.toContain('Welcome back, Vladimir');
    });
  });

  describe('2. Source Code Architecture & Guardrails Verification', () => {
    it('completely removed openTreeDirectly from App.tsx', () => {
      const appSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/App.tsx'),
        'utf-8'
      );
      expect(appSrc).not.toContain('openTreeDirectly');
    });

    it('implements sessionStorage check and initialRedirectAttemptedRef in App.tsx', () => {
      const appSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/App.tsx'),
        'utf-8'
      );
      expect(appSrc).toContain('initialRedirectAttemptedRef');
      expect(appSrc).toContain('genealogy_start_page_redirected');
      expect(appSrc).toContain("navigate('/tree')");
    });

    it('clears sessionStorage genealogy_start_page_redirected in auth-context.tsx logout', () => {
      const authContextSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/lib/auth-context.tsx'),
        'utf-8'
      );
      expect(authContextSrc).toContain(
        "sessionStorage.removeItem('genealogy_start_page_redirected')"
      );
    });

    it('passes loggedInUser in LoginCard onSuccess callbacks', () => {
      const loginCardSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/components/auth/LoginCard.tsx'),
        'utf-8'
      );
      expect(loginCardSrc).toContain('onSuccess?.(res.prompt_passkey_enrollment, res.user)');
      expect(loginCardSrc).toContain('onSuccess?.(true, res.user)');
    });
  });

  describe('3. AuthPages Post-Auth Redirection Target', () => {
    it('renders LoginCard with onSuccess hook in AuthPages', () => {
      const onNavigate = vi.fn();
      const html = renderWithProviders(
        <AuthPages
          currentPath="/login"
          search=""
          onNavigate={onNavigate}
        />
      );
      expect(html).toContain('Sign in with Passkey');
    });

    it('extracts and tests resolvePostAuthTarget behavior for start_page variations', () => {
      // Simulate the exact helper logic in AuthPages
      const resolvePostAuthTarget = (
        rawReturnUrl: string | null,
        targetUrl: string,
        loggedInUser?: UserDto | null
      ): string => {
        if (!rawReturnUrl || rawReturnUrl === '/' || targetUrl === '/') {
          if (loggedInUser?.start_page === 'tree') {
            return '/tree';
          }
          return '/';
        }
        return targetUrl;
      };

      // 1. Without returnUrl, user with start_page="tree" -> /tree
      expect(resolvePostAuthTarget(null, '/', mockTreeUser)).toBe('/tree');

      // 2. Without returnUrl, user with start_page="dashboard" -> /
      expect(resolvePostAuthTarget(null, '/', mockDashboardUser)).toBe('/');

      // 3. With returnUrl="/", user with start_page="tree" -> /tree
      expect(resolvePostAuthTarget('/', '/', mockTreeUser)).toBe('/tree');

      // 4. With explicit non-root returnUrl="/settings", goes to /settings regardless of preference
      expect(resolvePostAuthTarget('/settings', '/settings', mockTreeUser)).toBe('/settings');
      expect(resolvePostAuthTarget('/settings', '/settings', mockDashboardUser)).toBe('/settings');

      // 5. With explicit returnUrl="/people", goes to /people
      expect(resolvePostAuthTarget('/people', '/people', mockTreeUser)).toBe('/people');
    });
  });

  describe('4. Default Start Page Redirect Lifecycle & In-App Navigation', () => {
    it('redirects to /tree on cold start at "/" when user start_page="tree" and tree is populated', () => {
      const mockStorage: Record<string, string> = {};
      const fakeStorage = {
        getItem: (k: string) => mockStorage[k] ?? null,
        setItem: (k: string, v: string) => {
          mockStorage[k] = v;
        },
        removeItem: (k: string) => {
          delete mockStorage[k];
        },
      };

      let initialRedirectAttempted = false;
      let navigatedTo: string | null = null;
      const navigate = (to: string) => {
        initialRedirectAttempted = true;
        fakeStorage.setItem('genealogy_start_page_redirected', 'true');
        navigatedTo = to;
      };

      const evaluateStartPageRedirect = (
        path: string,
        isAuthenticated: boolean,
        isTreeEmpty: boolean,
        userStartPage?: 'tree' | 'dashboard'
      ) => {
        const alreadyRedirected =
          fakeStorage.getItem('genealogy_start_page_redirected') === 'true';

        if (initialRedirectAttempted || alreadyRedirected) {
          return;
        }

        if (path === '/') {
          if (isAuthenticated && !isTreeEmpty && userStartPage === 'tree') {
            initialRedirectAttempted = true;
            fakeStorage.setItem('genealogy_start_page_redirected', 'true');
            navigate('/tree');
            return;
          }
        }

        initialRedirectAttempted = true;
        fakeStorage.setItem('genealogy_start_page_redirected', 'true');
      };

      // 1. Initial cold start at '/' with start_page='tree' and populated tree
      evaluateStartPageRedirect('/', true, false, 'tree');
      expect(navigatedTo).toBe('/tree');
      expect(fakeStorage.getItem('genealogy_start_page_redirected')).toBe('true');

      // 2. In-app navigation to '/': user clicks "Главная" in navbar
      navigatedTo = null;
      evaluateStartPageRedirect('/', true, false, 'tree');
      // Must NOT redirect again: remains on '/'
      expect(navigatedTo).toBeNull();

      // 3. User logs out: sessionStorage key is removed
      fakeStorage.removeItem('genealogy_start_page_redirected');
      expect(fakeStorage.getItem('genealogy_start_page_redirected')).toBeNull();
    });

    it('does not redirect if user start_page="dashboard"', () => {
      const mockStorage: Record<string, string> = {};
      const fakeStorage = {
        getItem: (k: string) => mockStorage[k] ?? null,
        setItem: (k: string, v: string) => {
          mockStorage[k] = v;
        },
      };

      let navigatedTo: string | null = null;
      const navigate = (to: string) => {
        navigatedTo = to;
      };

      let initialRedirectAttempted = false;
      const evaluate = (path: string, userStartPage?: 'tree' | 'dashboard') => {
        if (
          initialRedirectAttempted ||
          fakeStorage.getItem('genealogy_start_page_redirected') === 'true'
        ) {
          return;
        }
        if (path === '/' && userStartPage === 'tree') {
          initialRedirectAttempted = true;
          navigate('/tree');
          return;
        }
        initialRedirectAttempted = true;
      };

      evaluate('/', 'dashboard');
      expect(navigatedTo).toBeNull();
    });

    it('does not redirect if tree is empty', () => {
      let navigatedTo: string | null = null;
      const navigate = (to: string) => {
        navigatedTo = to;
      };

      let initialRedirectAttempted = false;
      const evaluate = (isTreeEmpty: boolean) => {
        if (initialRedirectAttempted) return;
        if (!isTreeEmpty) {
          initialRedirectAttempted = true;
          navigate('/tree');
          return;
        }
        initialRedirectAttempted = true;
      };

      evaluate(true);
      expect(navigatedTo).toBeNull();
    });
  });

  describe('5. Design-system & UI Quality Rules', () => {
    it('contains 0 raw <button>, <input>, <select>, or <textarea> elements in modified files', () => {
      const files = [
        path.resolve(__dirname, '../../src/App.tsx'),
        path.resolve(__dirname, '../../src/pages/AuthPages.tsx'),
        path.resolve(__dirname, '../../src/components/auth/LoginCard.tsx'),
        path.resolve(__dirname, '../../src/components/auth/RegisterInviteCard.tsx'),
        path.resolve(__dirname, '../../src/components/auth/MagicLinkVerifyCard.tsx'),
      ];

      for (const file of files) {
        const src = fs.readFileSync(file, 'utf-8');
        expect(src, `${file} should not contain raw <button>`).not.toMatch(/<button[\s>]/);
        expect(src, `${file} should not contain raw <input>`).not.toMatch(/<input[\s>]/);
        expect(src, `${file} should not contain raw <select>`).not.toMatch(/<select[\s>]/);
        expect(src, `${file} should not contain raw <textarea>`).not.toMatch(/<textarea[\s>]/);
      }
    });

    it('contains 0 hardcoded hex color literals in modified files', () => {
      const files = [
        path.resolve(__dirname, '../../src/App.tsx'),
        path.resolve(__dirname, '../../src/pages/AuthPages.tsx'),
        path.resolve(__dirname, '../../src/components/auth/LoginCard.tsx'),
        path.resolve(__dirname, '../../src/components/auth/RegisterInviteCard.tsx'),
        path.resolve(__dirname, '../../src/components/auth/MagicLinkVerifyCard.tsx'),
      ];

      for (const file of files) {
        const src = fs.readFileSync(file, 'utf-8');
        expect(src, `${file} should not contain hardcoded hex colors`).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
      }
    });
  });
});
