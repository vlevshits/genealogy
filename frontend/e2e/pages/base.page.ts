import { Locator, Page } from '@playwright/test';

export class BasePage {
  readonly page: Page;
  readonly mobileMenuButton: Locator;
  readonly languageToggle: Locator;
  readonly themeToggle: Locator;
  readonly brandHeading: Locator;

  constructor(page: Page) {
    this.page = page;
    this.mobileMenuButton = page.locator('button:has(svg.lucide-menu)');
    this.languageToggle = page.locator('button:has(svg.lucide-globe)');
    this.themeToggle = page.locator(
      'button:has(svg.lucide-sun), button:has(svg.lucide-moon), button:has(svg.lucide-laptop)'
    );
    this.brandHeading = page.locator('header h1, header span').first();
  }

  async goto(path: string = '/') {
    await this.page.goto(path);
  }

  async ensureTogglesAccessible() {
    const isDesktopToggleVisible = await this.languageToggle.first().isVisible().catch(() => false);
    if (!isDesktopToggleVisible) {
      const isMobileMenuVisible = await this.mobileMenuButton.first().isVisible().catch(() => false);
      if (isMobileMenuVisible) {
        await this.mobileMenuButton.first().click();
        await this.page.locator('[role="dialog"]').first().waitFor({ state: 'visible', timeout: 5000 }).catch(() => {});
      }
    }
  }

  async toggleLanguage() {
    await this.ensureTogglesAccessible();
    await this.languageToggle.first().click();
  }

  async getCurrentLanguageText(): Promise<string> {
    await this.ensureTogglesAccessible();
    return (await this.languageToggle.first().innerText()).trim();
  }

  async toggleTheme() {
    await this.ensureTogglesAccessible();
    await this.themeToggle.first().click();
  }

  async isDarkMode(): Promise<boolean> {
    return await this.page.evaluate(() =>
      document.documentElement.classList.contains('dark')
    );
  }
}
