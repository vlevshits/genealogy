import { Locator, Page } from '@playwright/test';
import { BasePage } from './base.page';

export class AuthPage extends BasePage {
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly passkeyButton: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    super(page);
    this.emailInput = page.locator('input[type="email"]');
    this.passwordInput = page.locator('input[type="password"]');
    this.submitButton = page.locator('button[type="submit"]');
    this.passkeyButton = page.getByRole('button', { name: /Passkey|Ключ доступа/i });
    this.errorMessage = page.locator('[role="alert"]');
  }

  async gotoLogin(returnUrl?: string) {
    const path = returnUrl ? `/login?returnUrl=${encodeURIComponent(returnUrl)}` : '/login';
    await super.goto(path);
  }

  async loginWithPassword(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.submitButton.click();
  }
}
