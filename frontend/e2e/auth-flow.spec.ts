import { test, expect } from './fixtures/test-base';
import { AuthPage } from './pages/auth.page';

test.describe('Authentication & Access Guard Flows', () => {
  test.beforeEach(async ({ mockUnauthenticatedState }) => {
    await mockUnauthenticatedState();
  });

  test('redirects unauthenticated users from protected routes to login with returnUrl', async ({ page }) => {
    // Attempt visiting protected page
    await page.goto('/settings');

    // Should be redirected to /login with returnUrl query param
    await expect(page).toHaveURL(/\/login\?returnUrl=%2Fsettings/);
  });

  test('renders login card with credentials inputs and submit button', async ({ page }) => {
    const authPage = new AuthPage(page);
    await authPage.gotoLogin();

    await expect(authPage.emailInput).toBeVisible();
    await expect(authPage.passwordInput).toBeVisible();
    await expect(authPage.submitButton).toBeVisible();
  });

  test('keeps submit button disabled when credentials are empty', async ({ page }) => {
    const authPage = new AuthPage(page);
    await authPage.gotoLogin();

    // Verify submit button is disabled when email is empty
    await expect(authPage.submitButton).toBeDisabled();

    // Fill valid email and password
    await authPage.emailInput.fill('user@family.local');
    await authPage.passwordInput.fill('Pass1234!');

    // Submit button should now be enabled
    await expect(authPage.submitButton).toBeEnabled();
  });
});
