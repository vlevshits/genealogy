import { test as baseTest, expect, Page, Route } from '@playwright/test';

export interface MockUser {
  id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  display_name: string;
  role: 'admin' | 'editor' | 'viewer';
  start_page?: 'tree' | 'home' | 'directory' | 'dashboard';
}

export interface TestFixtures {
  /**
   * Helper to set language in localStorage before page loads.
   */
  setLocale: (locale: 'en' | 'ru') => Promise<void>;
  /**
   * Helper to set theme in localStorage.
   */
  setTheme: (theme: 'light' | 'dark' | 'system') => Promise<void>;
  /**
   * Helper to mock an unauthenticated visitor on a private initialized tree.
   */
  mockUnauthenticatedState: (options?: { enablePublicAccess?: boolean }) => Promise<void>;
  /**
   * Helper to mock an authenticated user session.
   */
  mockAuthenticatedState: (user?: Partial<MockUser>) => Promise<void>;
}

export const test = baseTest.extend<TestFixtures>({
  setLocale: async ({ page }, use) => {
    const setLocaleFn = async (locale: 'en' | 'ru') => {
      await page.addInitScript((loc) => {
        try {
          window.localStorage.setItem('genealogy_locale', loc);
        } catch {
          // Ignore
        }
      }, locale);
    };
    await use(setLocaleFn);
  },

  setTheme: async ({ page }, use) => {
    const setThemeFn = async (theme: 'light' | 'dark' | 'system') => {
      await page.addInitScript((th) => {
        try {
          window.localStorage.setItem('genealogy_theme', th);
        } catch {
          // Ignore
        }
      }, theme);
    };
    await use(setThemeFn);
  },

  mockUnauthenticatedState: async ({ page }, use) => {
    const mockFn = async (options: { enablePublicAccess?: boolean } = {}) => {
      const enablePublic = options.enablePublicAccess ?? false;

      const setupPayload = {
        is_initialized: true,
        tree_title: 'Levshits Family Archive',
        default_locale: 'en',
        enable_public_access: enablePublic,
        auth_password_enabled: true,
        smtp_enabled: true,
        rp_id: 'localhost',
        rp_name: 'Levshits Family Archive',
      };

      await page.route('**/api/v1/system/info', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            app_version: '0.1.0',
            instance_title: 'Levshits Family Archive',
            tree_title: 'Levshits Family Archive',
            default_locale: 'en',
            enable_public_access: enablePublic,
            auth_password_enabled: true,
            settings_locked_from_env: false,
            ai_enabled: true,
          }),
        });
      });

      await page.route(/.*\/api\/v1\/(auth\/)?setup\/status/, async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(setupPayload),
        });
      });

      await page.route('**/api/v1/auth/me', async (route) => {
        await route.fulfill({
          status: 401,
          contentType: 'application/json',
          body: JSON.stringify({
            error: {
              code: 'not_authenticated',
              message: 'Authentication required',
            },
          }),
        });
      });

      await page.route(/.*\/api\/v1\/(tree\/)?onboarding\/status/, async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            is_empty: false,
            person_count: 24,
            total_individuals: 24,
            can_onboard: false,
            resolved_home_person_id: 'node-root',
            default_home_person_id: 'node-root',
            user_home_person_id: null,
            user_person_id: null,
            start_page: 'dashboard',
          }),
        });
      });
    };

    await use(mockFn);
  },

  mockAuthenticatedState: async ({ page }, use) => {
    const mockFn = async (userOverrides: Partial<MockUser> = {}) => {
      const mockUser: MockUser = {
        id: 'usr-admin-01',
        email: 'admin@genealogy.local',
        first_name: 'Admin',
        last_name: 'User',
        display_name: 'Admin User',
        role: 'admin',
        start_page: 'home',
        ...userOverrides,
      };

      const setupPayload = {
        is_initialized: true,
        tree_title: 'Levshits Family Archive',
        default_locale: 'en',
        enable_public_access: false,
        auth_password_enabled: true,
        smtp_enabled: true,
        rp_id: 'localhost',
        rp_name: 'Levshits Family Archive',
      };

      await page.route('**/api/v1/system/info', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            app_version: '0.1.0',
            instance_title: 'Levshits Family Archive',
            tree_title: 'Levshits Family Archive',
            default_locale: 'en',
            enable_public_access: false,
            auth_password_enabled: true,
            settings_locked_from_env: false,
            ai_enabled: true,
          }),
        });
      });

      await page.route(/.*\/api\/v1\/(auth\/)?setup\/status/, async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(setupPayload),
        });
      });

      await page.route('**/api/v1/auth/me', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(mockUser),
        });
      });

      await page.route(/.*\/api\/v1\/(tree\/)?onboarding\/status/, async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            is_empty: false,
            person_count: 42,
            total_individuals: 42,
            can_onboard: false,
            resolved_home_person_id: 'node-root',
            default_home_person_id: 'node-root',
            user_home_person_id: 'node-root',
            user_person_id: 'node-root',
            start_page: 'dashboard',
          }),
        });
      });

      await page.route('**/api/v1/dashboard*', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            resolved_home_person_id: 'node-root',
            anniversaries: [],
            recent_activity: [],
            stats: {
              total_persons: 42,
              total_families: 14,
              total_photos: 12,
              total_places: 8,
              max_generations: 4,
              open_warnings_count: 0,
              missing_birth_dates_count: 0,
            },
            latest_photos: [],
          }),
        });
      });

      await page.route('**/api/v1/tree/view*', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            access_tier: mockUser.role,
            tree_title: 'Levshits Family Archive',
            focal_id: 'node-root',
            mode: 'hourglass',
            generations: 3,
            collateral: true,
            nodes: [
              {
                id: 'node-root',
                gid: 'I0001',
                sex: 'F',
                is_living: true,
                is_private: false,
                primary_name: {
                  given_name: 'Anna',
                  patronymic: 'Sergeyevna',
                  surname: 'Volkova',
                },
                birth: { date: '1975', place: 'Tver' },
                death: null,
                avatar_url: null,
                notes: null,
                version: 1,
                generation: 0,
                is_direct_lineage: true,
              },
            ],
            families: [],
            edges: [],
            resolved_home_person_id: 'node-root',
            total_visible: 1,
          }),
        });
      });
    };

    await use(mockFn);
  },

  page: async ({ page }, use) => {
    const errors: string[] = [];

    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Ignore expected network connection errors or benign dev warnings
        if (
          !text.includes('Failed to load resource') &&
          !text.includes('ECONNREFUSED') &&
          !text.includes('ERR_CONNECTION_REFUSED') &&
          !text.includes('401')
        ) {
          errors.push(text);
        }
      }
    });

    page.on('pageerror', (exception) => {
      errors.push(`Unhandled exception: ${exception.message}`);
    });

    await use(page);

    // Assert no unexpected uncaught JS runtime errors
    expect(
      errors,
      `Unexpected console/runtime errors encountered:\n${errors.join('\n')}`
    ).toEqual([]);
  },
});

export { expect };
