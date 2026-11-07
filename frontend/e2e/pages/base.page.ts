import { Locator, Page } from '@playwright/test';

export class BasePage {
  readonly page: Page;
  readonly mobileMenuButton: Locator;
  readonly languageToggle: Locator;
  readonly themeToggle: Locator;
  readonly brandHeading: Locator;

  constructor(page: Page) {
    this.page = page;
    this.mobileMenuButton = page
      .getByTestId('mobile-menu-toggle')
      .or(page.getByRole('button', { name: /Open navigation menu|Открыть меню навигации/i }));
    this.languageToggle = page
      .getByTestId('language-toggle')
      .or(page.getByRole('button', { name: /^(Language|Язык)$/i }));
    this.themeToggle = page
      .getByTestId('theme-toggle')
      .or(page.getByRole('button', { name: /Select color theme|Выбрать тему оформления/i }));
    this.brandHeading = page.locator('header h1, header span').first();
  }

  async goto(path: string = '/') {
    await this.page.goto(path);
  }

  async ensureTogglesAccessible() {
    const isDesktopToggleVisible = await this.languageToggle.first().isVisible();
    if (!isDesktopToggleVisible) {
      const isMobileMenuVisible = await this.mobileMenuButton.first().isVisible();
      if (isMobileMenuVisible) {
        await this.mobileMenuButton.first().click();
        await this.page.getByRole('dialog').first().waitFor({ state: 'visible', timeout: 5000 });
      }
    }
  }

  async toggleLanguage() {
    await this.ensureTogglesAccessible();
    const target = (await this.languageToggle.first().isVisible())
      ? this.languageToggle.first()
      : this.languageToggle.last();
    await target.click();
  }

  async getCurrentLanguageText(): Promise<string> {
    await this.ensureTogglesAccessible();
    const target = (await this.languageToggle.first().isVisible())
      ? this.languageToggle.first()
      : this.languageToggle.last();
    return (await target.innerText()).trim();
  }

  async toggleTheme() {
    await this.ensureTogglesAccessible();
    const target = (await this.themeToggle.first().isVisible())
      ? this.themeToggle.first()
      : this.themeToggle.last();
    await target.click();
  }

  async isDarkMode(): Promise<boolean> {
    return await this.page.evaluate(() =>
      document.documentElement.classList.contains('dark')
    );
  }
}
