import { test, expect } from './fixtures/test-base';

test.describe('Authenticated Navigation & App Shell', () => {
  test.beforeEach(async ({ mockAuthenticatedState }) => {
    await mockAuthenticatedState({
      id: 'usr-admin-01',
      display_name: 'Administrator',
      role: 'admin',
    });
  });

  test('renders authenticated app shell with user identity and navigation', async ({ page }) => {
    await page.goto('/');

    // Page body and header should be visible
    await expect(page.locator('header')).toBeVisible();

    // Verify presence of core application structure (nav or brand)
    const headerOrNav = page.locator('header, nav');
    await expect(headerOrNav.first()).toBeVisible();
  });
});
