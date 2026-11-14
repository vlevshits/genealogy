import { expect, type Locator, type Page } from '@playwright/test';
import { BasePage } from './base.page';
import { tr, type Locale } from '../fixtures/i18n';

const UUID_REGEX =
  /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i;

/**
 * Resolve ICU `{count, plural, one {...} few {...} many {...} other {...}}`
 * templates matching `frontend/src/i18n/index.tsx` Russian/English rules.
 */
export function resolveIcuTemplate(
  template: string,
  params: Record<string, string | number>
): string {
  const selectPlural = (n: number, forms: Record<string, string>): string => {
    const abs = Math.abs(n);
    const mod10 = abs % 10;
    const mod100 = abs % 100;
    let cat = 'other';
    if (abs === 1 && forms.one && !forms.few && !forms.many) {
      cat = 'one';
    } else if (forms.few || forms.many) {
      if (mod10 === 1 && mod100 !== 11) cat = 'one';
      else if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
        cat = 'few';
      } else cat = 'many';
    } else if (abs === 1) {
      cat = 'one';
    }
    return forms[cat] ?? forms.other ?? Object.values(forms)[0] ?? '';
  };

  let out = '';
  let i = 0;
  while (i < template.length) {
    if (template[i] !== '{') {
      out += template[i++];
      continue;
    }
    let depth = 0;
    let j = i;
    while (j < template.length) {
      if (template[j] === '{') depth++;
      else if (template[j] === '}') {
        depth--;
        if (depth === 0) break;
      }
      j++;
    }
    if (depth !== 0) {
      out += template.slice(i);
      break;
    }
    const inner = template.slice(i + 1, j);
    const comma1 = inner.indexOf(',');
    if (comma1 === -1) {
      const val = params[inner.trim()];
      out += val !== undefined ? String(val) : `{${inner}}`;
      i = j + 1;
      continue;
    }
    const varName = inner.slice(0, comma1).trim();
    const rest = inner.slice(comma1 + 1);
    const comma2 = rest.indexOf(',');
    const icuType = (comma2 === -1 ? rest : rest.slice(0, comma2)).trim();
    if (icuType !== 'plural' || comma2 === -1) {
      const val = params[varName];
      out += val !== undefined ? String(val) : `{${inner}}`;
      i = j + 1;
      continue;
    }
    const formsStr = rest.slice(comma2 + 1);
    const forms: Record<string, string> = {};
    let k = 0;
    while (k < formsStr.length) {
      while (k < formsStr.length && /\s/.test(formsStr[k])) k++;
      const keyStart = k;
      while (
        k < formsStr.length &&
        !/[\s{]/.test(formsStr[k])
      ) {
        k++;
      }
      const key = formsStr.slice(keyStart, k);
      while (k < formsStr.length && /\s/.test(formsStr[k])) k++;
      if (!key || formsStr[k] !== '{') break;
      let fDepth = 0;
      const bodyStart = k + 1;
      while (k < formsStr.length) {
        if (formsStr[k] === '{') fDepth++;
        else if (formsStr[k] === '}') {
          fDepth--;
          if (fDepth === 0) break;
        }
        k++;
      }
      forms[key] = formsStr.slice(bodyStart, k);
      k++;
    }
    const numVal = Number(params[varName] ?? 0);
    const chosen = selectPlural(numVal, forms).replace(/#/g, String(numVal));
    out += resolveIcuTemplate(chosen, params);
    i = j + 1;
  }
  return out;
}

export class HistoryVisualizationPage extends BasePage {
  readonly pageRoot: Locator;
  readonly filterBar: Locator;
  readonly errorToast: Locator;
  readonly successToast: Locator;
  readonly drawerContainer: Locator;
  readonly conflictModalContent: Locator;

  constructor(page: Page) {
    super(page);
    this.pageRoot = page.getByTestId('history-page');
    this.filterBar = page.getByTestId('history-filter-bar');
    this.errorToast = page.getByTestId('toast-error');
    this.successToast = page.getByTestId('toast-success');
    this.drawerContainer = page.getByTestId('record-history-drawer');
    this.conflictModalContent = page.getByTestId('conflict-resolution-modal');
  }

  async openHistoryPage(locale: Locale = 'en'): Promise<void> {
    await this.goto('/history');
    await expect(this.pageRoot).toBeVisible();
    await expect(
      this.page.getByRole('heading', { name: tr(locale, 'history.page.title') })
    ).toBeVisible();
  }

  async openPersonProfile(personId: string, locale: Locale = 'en'): Promise<void> {
    await this.goto(`/persons/${personId}`);
    await expect(
      this.page.getByRole('region', { name: tr(locale, 'profile.hero.ariaLabel') })
    ).toBeVisible();
  }

  txCard(txId: string): Locator {
    return this.page.getByTestId(`history-tx-${txId}`);
  }

  txLabel(txId: string): Locator {
    return this.page.getByTestId(`history-tx-label-${txId}`);
  }

  undoButton(txId: string): Locator {
    return this.page.getByTestId(`history-undo-tx-${txId}`);
  }

  restoreDeletedButton(entryId: string | number): Locator {
    return this.page.getByTestId(`history-restore-entry-${entryId}`);
  }

  entityTypeCombobox(locale: Locale): Locator {
    return this.filterBar.getByRole('combobox', {
      name: tr(locale, 'history.filters.entityTypeLabel'),
    });
  }

  actionCombobox(locale: Locale): Locator {
    return this.filterBar.getByRole('combobox', {
      name: tr(locale, 'history.filters.actionLabel'),
    });
  }

  onlyDeletionsSwitch(locale: Locale): Locator {
    return this.filterBar.getByRole('switch', {
      name: tr(locale, 'history.filters.onlyDeletionsLabel'),
    });
  }

  fromDateInput(locale: Locale): Locator {
    return this.filterBar.getByLabel(tr(locale, 'history.filters.fromDateLabel'));
  }

  toDateInput(locale: Locale): Locator {
    return this.filterBar.getByLabel(tr(locale, 'history.filters.toDateLabel'));
  }

  conflictErrorAlert(): Locator {
    return this.conflictModalContent.getByTestId('conflict-error-alert');
  }

  async ensureFiltersExpanded(): Promise<void> {
    const toggle = this.page.getByTestId('history-mobile-filters-toggle');
    if (await toggle.isVisible()) {
      const expanded = await toggle.getAttribute('aria-expanded');
      if (expanded !== 'true') {
        await toggle.click();
      }
    }
  }

  async selectFilterOption(combobox: Locator, optionText: string): Promise<void> {
    await this.ensureFiltersExpanded();
    await combobox.click();
    const listbox = this.page.getByRole('listbox');
    await expect(listbox).toBeVisible();
    await listbox.getByRole('option', { name: optionText, exact: true }).click();
  }

  recordHistoryDrawerDialog(locale: Locale): Locator {
    return this.page.getByRole('dialog', {
      name: tr(locale, 'history.drawer.title'),
    });
  }

  drawerEntry(entryId: string | number): Locator {
    return this.drawerContainer.getByTestId(`history-entry-${entryId}`);
  }

  restoreConfirmDialog(locale: Locale): Locator {
    return this.page.getByRole('dialog', {
      name: tr(locale, 'history.restore.confirmTitle'),
    });
  }

  personEditDialog(locale: Locale): Locator {
    return this.page.getByRole('dialog', {
      name: tr(locale, 'person.edit.title'),
    });
  }

  conflictDialog(locale: Locale): Locator {
    return this.page.getByRole('dialog', {
      name: tr(locale, 'history.conflict.title'),
    });
  }

  conflictFieldRow(field: string): Locator {
    return this.conflictModalContent.getByTestId(`conflict-field-${field}`);
  }

  async openRecordHistoryDrawer(locale: Locale): Promise<void> {
    const hero = this.page.getByRole('region', {
      name: tr(locale, 'profile.hero.ariaLabel'),
    });
    const overflowTrigger = hero.getByRole('button', {
      name: tr(locale, 'profile.actions.moreAria'),
    });
    await overflowTrigger.click();
    const menu = this.page.getByRole('menu', {
      name: tr(locale, 'profile.actions.moreAria'),
    });
    await expect(menu).toBeVisible();
    await menu
      .getByRole('menuitem', { name: tr(locale, 'profile.actions.viewHistory') })
      .click();
    await expect(this.drawerContainer).toBeVisible();
  }

  async openPersonEditDialog(locale: Locale): Promise<Locator> {
    await this.page
      .getByRole('button', { name: tr(locale, 'profile.actions.editProfile') })
      .click();
    await this.page
      .getByRole('button', {
        name: tr(locale, 'profile.actions.fullIdentityDialog'),
      })
      .click();
    const dialog = this.personEditDialog(locale);
    await expect(dialog).toBeVisible();
    return dialog;
  }

  async assertNoUuidOrJsonLeaks(container: Locator): Promise<void> {
    const text = await container.innerText();
    expect(text, 'Rendered UI text must not contain 36-char UUIDs').not.toMatch(
      UUID_REGEX
    );
    expect(text, 'Rendered UI text must not contain raw JSON objects').not.toContain(
      '{"'
    );
    expect(text, 'Rendered UI text must not contain [object Object]').not.toContain(
      '[object Object]'
    );
  }

  async assertNoRawI18nLeaks(container: Locator): Promise<void> {
    const text = await container.innerText();
    expect(
      text,
      'Rendered UI text must not expose unresolved history.* or domain.* keys'
    ).not.toMatch(/\b(?:history|domain|person|profile|rbac)\.[a-z0-9_.]+/i);
    expect(
      text,
      'Rendered UI text must not expose humanized raw operation code Person.update Profile'
    ).not.toContain('Person.update Profile');
  }

  async assertNoHorizontalOverflow(): Promise<void> {
    const overflow = await this.page.evaluate(() => ({
      scrollWidth: document.documentElement.scrollWidth,
      innerWidth: window.innerWidth,
    }));
    expect(
      overflow.scrollWidth,
      `scrollWidth (${overflow.scrollWidth}) must be <= innerWidth (${overflow.innerWidth})`
    ).toBeLessThanOrEqual(overflow.innerWidth);
  }
}

