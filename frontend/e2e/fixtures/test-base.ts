import { test as baseTest, expect, Page, Route } from '@playwright/test';

export interface MockUser {
  id: string;
  email: string;
  display_name: string;
  role: 'admin' | 'editor' | 'viewer';
  start_page?: 'tree' | 'home' | 'directory';
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
            default_locale: 'en',
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
            code: 'AUTH_REQUIRED',
            message: 'Authentication required',
          }),
        });
      });

      await page.route('**/api/v1/tree/onboarding/status', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            is_empty: false,
            person_count: 24,
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
            default_locale: 'en',
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

      await page.route('**/api/v1/tree/onboarding/status', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            is_empty: false,
            person_count: 42,
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
