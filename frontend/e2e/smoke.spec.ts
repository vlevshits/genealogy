import { test, expect } from './fixtures/test-base';
import { BasePage } from './pages/base.page';

test.describe('Smoke & Core UI Functionality', () => {
  test.beforeEach(async ({ mockUnauthenticatedState }) => {
    await mockUnauthenticatedState();
  });

  test('loads home/login page and displays brand elements', async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto('/');

    // Verify page loads and displays header
    await expect(page.locator('body')).toBeVisible();
    await expect(basePage.languageToggle.first()).toBeVisible();
    await expect(basePage.themeToggle.first()).toBeVisible();
  });

  test('toggles language between English and Russian', async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto('/');

    // Initially in EN or RU; click toggle and verify language text switch
    const initialLang = await basePage.getCurrentLanguageText();
    expect(['EN', 'RU']).toContain(initialLang);

    await basePage.toggleLanguage();
    const nextLang = await basePage.getCurrentLanguageText();
    expect(nextLang).not.toBe(initialLang);
    expect(['EN', 'RU']).toContain(nextLang);

    // Verify localStorage was updated
    const savedLang = await page.evaluate(() =>
      window.localStorage.getItem('genealogy_locale')
    );
    expect(savedLang).toBe(nextLang.toLowerCase());
  });

  test('toggles theme between light and dark modes', async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto('/');

    // Cycle through themes
    await basePage.toggleTheme();

    // After clicking theme toggle, state should advance in localStorage
    const savedTheme = await page.evaluate(() =>
      window.localStorage.getItem('genealogy_theme')
    );
    expect(['light', 'dark', 'system']).toContain(savedTheme);
  });
});
