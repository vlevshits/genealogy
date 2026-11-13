import { Locator, Page } from '@playwright/test';
import { BasePage } from './base.page';
import { exact, Locale, tr } from '../fixtures/i18n';

/** Person profile page: Family & Relations tab (link management). */
export class PersonProfilePage extends BasePage {
  readonly locale: Locale;
  readonly familyTab: Locator;
  readonly dialog: Locator;
  readonly undoButton: Locator;
  readonly unlinkSpouseButton: Locator;
  readonly unlinkChildButton: Locator;
  readonly unlinkParentButtons: Locator;
  readonly changeParentsButton: Locator;

  constructor(page: Page, locale: Locale) {
    super(page);
    this.locale = locale;
    this.familyTab = page.getByRole('tab', { name: this.t('profile.tabs.family') });
    this.dialog = page.getByRole('dialog');
    this.undoButton = page
      .getByRole('status')
      .getByRole('button', { name: exact(this.t('common.undo')) });
    this.unlinkSpouseButton = page.getByRole('button', {
      name: this.t('profile.family.removeSpouseAria'),
    });
    this.unlinkChildButton = page.getByRole('button', {
      name: this.t('profile.family.removeChildAria'),
    });
    this.unlinkParentButtons = page.getByRole('button', {
      name: this.t('profile.family.unlinkParentAria'),
    });
    this.changeParentsButton = page.getByRole('button', {
      name: exact(this.t('profile.family.changeParentsBtn')),
    });
  }

  t(key: string, params?: Record<string, string>): string {
    return tr(this.locale, key, params);
  }

  async open(personId: string) {
    await this.goto(`/persons/${personId}`);
    await this.familyTab.click();
  }

  /** Relative card text locator (name is rendered inside the Family tab). */
  person(name: string): Locator {
    return this.page.getByText(name, { exact: true });
  }

  toast(text: string): Locator {
    return this.page.getByRole('status').filter({ hasText: text });
  }

  async confirmRemoval() {
    await this.dialog
      .getByRole('button', { name: exact(this.t('profile.family.removeConfirm')) })
      .click();
  }

  async undo() {
    await this.undoButton.click();
  }

  quickAddButton(key: 'addSibling' | 'addSpouse' | 'addChild'): Locator {
    return this.page.getByRole('button', { name: exact(this.t(`person.actions.${key}`)) });
  }

  // --- Quick-add dialog -----------------------------------------------------

  async switchQuickAddToLinkExisting() {
    await this.dialog
      .getByRole('radio', { name: exact(this.t('person.quickAdd.modeLinkExisting')) })
      .click();
  }

  existingPersonCombobox(): Locator {
    return this.dialog.getByRole('combobox', {
      name: this.t('person.quickAdd.selectExistingLabel'),
    });
  }

  /** Listbox options currently offered by the open Combobox. */
  options(): Locator {
    return this.page.getByRole('option');
  }

  noResults(): Locator {
    return this.page.getByText(this.t('common.noResults'), { exact: true });
  }

  // --- Change parents dialog ------------------------------------------------

  async pickParent(slot: 'parent1Label' | 'parent2Label', search: string, optionName: string) {
    const combo = this.dialog.getByRole('combobox', { name: this.t(`profile.family.${slot}`) });
    await combo.click();
    await combo.fill(search);
    await this.page.getByRole('option', { name: new RegExp(optionName) }).click();
  }

  async submitChangeParents() {
    await this.dialog
      .getByRole('button', { name: exact(this.t('profile.family.changeParentsSubmit')) })
      .click();
  }
}
