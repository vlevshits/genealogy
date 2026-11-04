import { Locator, Page } from '@playwright/test';
import { BasePage } from './base.page';

export class WorkbenchPage extends BasePage {
  readonly pageHeader: Locator;
  readonly openDialogButton: Locator;
  readonly dialog: Locator;
  readonly closeDialogButton: Locator;

  constructor(page: Page) {
    super(page);
    this.pageHeader = page.getByRole('heading', { level: 1 }).first();
    this.openDialogButton = page.getByRole('button', { name: /Medium Dialog|Среднее окно/i });
    this.dialog = page.getByRole('dialog');
    this.closeDialogButton = page.getByRole('button', { name: /Cancel|Отмена/i });
  }

  async goto() {
    await super.goto('/dev/ui');
  }
}
