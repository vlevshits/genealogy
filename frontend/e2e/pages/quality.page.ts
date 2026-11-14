import { expect, Locator, Page } from '@playwright/test';
import { BasePage } from './base.page';
import { Locale, exact, tr as trBase } from '../fixtures/i18n';

function getPluralCategory(
  locale: Locale,
  count: number
): 'one' | 'few' | 'many' | 'other' {
  const abs = Math.abs(count);
  if (locale === 'ru') {
    const mod10 = abs % 10;
    const mod100 = abs % 100;
    if (mod10 === 1 && mod100 !== 11) return 'one';
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return 'few';
    return 'many';
  }
  return abs === 1 ? 'one' : 'other';
}

function resolveIcuTemplate(
  locale: Locale,
  template: string,
  params: Record<string, string | number>
): string {
  const pluralRegex = /\{(\w+),\s*plural,\s*((?:[a-z]+\s*\{[^}]*\}\s*)+)\}/g;
  const resolved = template.replace(
    pluralRegex,
    (_match, varName: string, body: string) => {
      const rawVal = params[varName];
      const numVal = typeof rawVal === 'number' ? rawVal : Number(rawVal ?? 0);
      const category = getPluralCategory(locale, numVal);

      const branchRegex = /([a-z]+)\s*\{([^}]*)\}/g;
      const branches: Record<string, string> = {};
      let m: RegExpExecArray | null = branchRegex.exec(body);
      while (m) {
        branches[m[1]] = m[2];
        m = branchRegex.exec(body);
      }

      const chosen = branches[category] ?? branches.other ?? branches.many ?? '';
      return chosen.replace(/#/g, String(numVal));
    }
  );

  return resolved.replace(/\{(\w+)\}/g, (_m, key: string) =>
    key in params ? String(params[key]) : `{${key}}`
  );
}

/**
 * Page Object Model for `/quality` (Data Quality & Duplicate Events).
 */
export class QualityPage extends BasePage {
  locale: Locale;
  readonly pageRoot: Locator;
  readonly duplicateEventsSection: Locator;
  readonly batchMergeButton: Locator;
  readonly dialog: Locator;
  readonly mobileNavDrawer: Locator;
  readonly undoToast: Locator;
  readonly successToast: Locator;
  readonly errorToast: Locator;

  constructor(page: Page, locale: Locale = 'en') {
    super(page);
    this.locale = locale;
    this.pageRoot = page.getByTestId('data-quality-page');
    this.duplicateEventsSection = page.getByTestId('duplicate-events-section');
    this.batchMergeButton = page.getByTestId('batch-merge-events-btn');
    this.dialog = page.getByRole('dialog');
    this.mobileNavDrawer = page.locator('aside[role="dialog"]');
    this.undoToast = page.getByTestId('toast-undo');
    this.successToast = page.getByTestId('toast-success');
    this.errorToast = page.getByTestId('toast-error');
  }

  t(key: string, params: Record<string, string | number> = {}): string {
    const template = trBase(this.locale, key);
    return resolveIcuTemplate(this.locale, template, params);
  }

  get filterGroup(): Locator {
    return this.pageRoot.getByRole('radiogroup', {
      name: exact(this.t('quality.filterAria')),
    });
  }

  duplicatesTab(count?: number): Locator {
    if (count !== undefined) {
      return this.filterGroup.getByRole('radio', {
        name: exact(this.t('quality.tabs.duplicates', { count })),
      });
    }
    return this.filterGroup.getByRole('radio', {
      name: /^(Duplicates|Дубликаты)\s*\(\d+\)$/i,
    });
  }

  summaryWarningsBadge(count: number): Locator {
    return this.pageRoot.getByText(
      exact(this.t('quality.summaryWarnings', { count }))
    );
  }

  summaryDuplicatesBadge(count: number): Locator {
    return this.pageRoot.getByText(
      exact(this.t('quality.summaryDuplicates', { count }))
    );
  }

  summaryDuplicateEventsBadge(count: number): Locator {
    return this.pageRoot.getByText(
      exact(this.t('quality.summaryDuplicateEvents', { count }))
    );
  }

  get allEventCards(): Locator {
    return this.duplicateEventsSection.locator(
      '[data-testid^="duplicate-event-card-"]'
    );
  }

  eventCard(groupId: string): Locator {
    return this.page.getByTestId(`duplicate-event-card-${groupId}`);
  }

  mergeButton(groupId: string): Locator {
    return this.page.getByTestId(`merge-duplicate-event-btn-${groupId}`);
  }

  customizeButton(groupId: string): Locator {
    return this.page.getByTestId(`customize-duplicate-event-btn-${groupId}`);
  }

  dismissButton(groupId: string): Locator {
    return this.page.getByTestId(`dismiss-duplicate-event-btn-${groupId}`);
  }

  warningReviewDupEventsButton(groupId: string): Locator {
    return this.page.getByTestId(`warning-review-dup-events-btn-${groupId}`);
  }

  get emptyEventsHeading(): Locator {
    return this.duplicateEventsSection.getByText(
      exact(this.t('quality.duplicateEvents.emptyTitle'))
    );
  }

  keepEventRadio(eventId: string): Locator {
    return this.dialog.getByTestId(`customize-keep-event-${eventId}`);
  }

  get dateInput(): Locator {
    return this.dialog.getByTestId('customize-event-date-input');
  }

  get placeInput(): Locator {
    return this.dialog.getByTestId('customize-event-place-input');
  }

  get descriptionInput(): Locator {
    return this.dialog.getByTestId('customize-event-description-input');
  }

  get sourceInput(): Locator {
    return this.dialog.getByTestId('customize-event-source-input');
  }

  quickFillButton(value: string): Locator {
    return this.dialog.getByRole('button', { name: exact(value) });
  }

  get combineSourcesChip(): Locator {
    return this.dialog.getByTestId('combine-sources-chip');
  }

  get confirmCustomizeButton(): Locator {
    return this.dialog.getByTestId('confirm-customize-merge-event-btn');
  }

  get cancelCustomizeButton(): Locator {
    return this.dialog.getByRole('button', {
      name: exact(this.t('common.cancel')),
    });
  }

  get undoButton(): Locator {
    return this.undoToast.getByRole('button', {
      name: exact(this.t('common.undo')),
    });
  }

  toast(title: string): Locator {
    return this.page.getByRole('status').filter({ hasText: title });
  }

  async open(path: string = '/quality') {
    await this.goto(path);
    await expect(this.pageRoot).toBeVisible();
    await expect(this.duplicateEventsSection).toBeVisible();
  }

  async openDuplicatesTab(expectedCount?: number) {
    const tab = this.duplicatesTab(expectedCount);
    await tab.click();
    await expect(tab).toHaveAttribute('aria-checked', 'true');
    await expect(this.duplicateEventsSection).toBeVisible();
  }

  async openCustomizeModal(groupId: string) {
    await this.customizeButton(groupId).click();
    await expect(this.dialog).toBeVisible();
    await expect(this.confirmCustomizeButton).toBeVisible();
  }

  async undo() {
    await expect(this.undoToast).toBeVisible();
    await this.undoButton.click();
  }

  async switchLocaleLive(nextLocale: Locale) {
    await this.toggleLanguage();
    this.locale = nextLocale;
    // On mobile (<768px), toggleLanguage opens the AppShell navigation Drawer.
    // Close it via Escape so assertions check the page cleanly.
    if (await this.mobileNavDrawer.isVisible()) {
      await this.page.keyboard.press('Escape');
      await expect(this.mobileNavDrawer).toBeHidden();
    }
    await expect(
      this.pageRoot.getByRole('heading', {
        name: exact(this.t('quality.pageTitle')),
      })
    ).toBeVisible();
  }

  async assertUndoToastNotClipped() {
    await expect(this.undoToast).toBeVisible();
    const metrics = await this.undoToast.evaluate((el) => {
      const vw = window.innerWidth;
      const rect = el.getBoundingClientRect();
      const titleEl = el.querySelector('.font-medium');
      const descEl = el.querySelector('.text-xs.text-secondary');
      const checks = [titleEl, descEl]
        .filter((node): node is Element => Boolean(node))
        .map((node) => {
          const cs = getComputedStyle(node);
          return {
            text: (node.textContent || '').trim(),
            overflowX: node.scrollWidth - node.clientWidth,
            textOverflow: cs.textOverflow,
            whiteSpace: cs.whiteSpace,
          };
        });
      return {
        left: rect.left,
        right: rect.right,
        vw,
        checks,
      };
    });
    expect(metrics.left).toBeGreaterThanOrEqual(-1);
    expect(metrics.right).toBeLessThanOrEqual(metrics.vw + 1);
    for (const c of metrics.checks) {
      expect(c.textOverflow, `Expected no ellipsis on "${c.text}"`).not.toBe('ellipsis');
      expect(c.whiteSpace, `Expected multi-line wrapping on "${c.text}"`).not.toBe('nowrap');
      expect(
        c.overflowX,
        `Expected zero horizontal clipping on toast text "${c.text}"`
      ).toBeLessThanOrEqual(1);
    }
  }

  async assertFilterTabsSingleLineNoClip() {
    const issues = await this.filterGroup.evaluate((group) => {
      const vw = window.innerWidth;
      const groupRect = group.getBoundingClientRect();
      const radios = Array.from(group.querySelectorAll('[role="radio"]'));
      const problems: string[] = [];

      if (groupRect.right > vw + 1 || groupRect.left < -1) {
        problems.push(
          `radiogroup clipped: left=${groupRect.left.toFixed(1)} right=${groupRect.right.toFixed(1)} vw=${vw}`
        );
      }

      for (const btn of radios) {
        const text = (btn.textContent || '').trim();
        const r = btn.getBoundingClientRect();
        if (r.right > vw + 1 || r.left < -1) {
          problems.push(`tab "${text}" clipped outside viewport (right=${r.right.toFixed(1)}, vw=${vw})`);
        }
        if (btn.scrollWidth - btn.clientWidth > 1) {
          problems.push(`tab "${text}" horizontal overflow (${btn.scrollWidth} > ${btn.clientWidth})`);
        }
        const labelSpan = btn.querySelector('span');
        if (labelSpan) {
          const range = document.createRange();
          range.selectNodeContents(labelSpan);
          const rectTops = Array.from(range.getClientRects()).map((cr) => cr.top);
          if (
            rectTops.length > 1 &&
            Math.max(...rectTops) - Math.min(...rectTops) > 3
          ) {
            problems.push(`tab "${text}" wrapped across multiple lines`);
          }
        }
      }
      return problems;
    });
    expect(issues, `Filter tabs layout issues: ${issues.join('; ')}`).toEqual([]);
  }

  async assertNoRawI18nLeaks() {
    const text = await this.pageRoot.innerText();
    expect(text).not.toMatch(/\b(quality|domain\.events|common|history)\.[a-zA-Z0-9_.-]+/);
    expect(text).not.toMatch(/\{[a-zA-Z_]+,\s*plural,/);
    expect(text).not.toMatch(/\{(count|gid|events|score)\}/);
  }

  async assertResponsiveNoOverflow(groupIds: string[]) {
    const metrics = await this.page.evaluate((ids) => {
      const docClientWidth = document.documentElement.clientWidth;
      const docScrollWidth = document.documentElement.scrollWidth;
      const docOverflow = docScrollWidth - docClientWidth;
      const vw = window.innerWidth;
      const clipped: string[] = [];
      const offending: string[] = [];

      if (docOverflow > 1) {
        const all = document.querySelectorAll('*');
        for (const el of Array.from(all)) {
          if (el.closest('.overflow-x-auto') !== null) continue;
          const r = el.getBoundingClientRect();
          if (r.right > docClientWidth + 1 || r.left < -1) {
            offending.push(
              `${el.tagName.toLowerCase()}${el.getAttribute('data-testid') ? `[data-testid=${el.getAttribute('data-testid')}]` : ''}(L=${r.left.toFixed(0)},R=${r.right.toFixed(0)},W=${r.width.toFixed(0)},scroll=${el.scrollWidth},txt=${(el.textContent || '').trim().slice(0, 24)})`
            );
          }
        }
      }

      const checkEl = (el: Element | null, label: string) => {
        if (!el) return;
        const rect = el.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) {
          clipped.push(`${label}: zero-size`);
          return;
        }
        if (rect.left < -1 || rect.right > vw + 1) {
          clipped.push(
            `${label}: horizontal clip left=${rect.left.toFixed(1)} right=${rect.right.toFixed(1)} vw=${vw}`
          );
        }
      };

      checkEl(
        document.querySelector('[data-testid="batch-merge-events-btn"]'),
        'batch-merge-events-btn'
      );

      for (const id of ids) {
        checkEl(
          document.querySelector(`[data-testid="duplicate-event-card-${id}"]`),
          `card-${id}`
        );
        checkEl(
          document.querySelector(`[data-testid="dismiss-duplicate-event-btn-${id}"]`),
          `dismiss-${id}`
        );
        checkEl(
          document.querySelector(`[data-testid="customize-duplicate-event-btn-${id}"]`),
          `customize-${id}`
        );
        checkEl(
          document.querySelector(`[data-testid="merge-duplicate-event-btn-${id}"]`),
          `merge-${id}`
        );
      }

      return { docOverflow, vw, clipped, offending: offending.slice(0, 10) };
    }, groupIds);

    expect(
      metrics.docOverflow,
      `Expected no horizontal overflow on viewport (vw=${metrics.vw}), got ${metrics.docOverflow}px. Offending: ${metrics.offending.join(' | ')}`
    ).toBeLessThanOrEqual(1);
    expect(
      metrics.clipped,
      `Found clipped controls: ${metrics.clipped.join('; ')}`
    ).toEqual([]);
  }
}
