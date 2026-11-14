import { test, expect } from './fixtures/test-base';
import { Locale, exact } from './fixtures/i18n';
import {
  BIRTH_SAFE_GROUP,
  DEATH_SAFE_GROUP,
  MARRIAGE_CONFLICT_GROUP,
  installQualityApiMock,
} from './fixtures/quality-api-mock';
import { QualityPage } from './pages/quality.page';

const LOCALES: Locale[] = ['en', 'ru'];

for (const locale of LOCALES) {
  test.describe(`Data Quality — Duplicate Events (${locale})`, () => {
    test.beforeEach(async ({ setLocale, mockAuthenticatedState }) => {
      await setLocale(locale);
      await mockAuthenticatedState({ role: 'admin' });
    });

    test('renders summary bar, tab counts, individual birth and couple marriage cards', async ({
      page,
    }) => {
      await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality');

      // Summary bar counts are distinct and reflect mocked API payloads.
      await expect(quality.summaryWarningsBadge(1)).toBeVisible();
      await expect(quality.summaryDuplicatesBadge(1)).toBeVisible();
      await expect(quality.summaryDuplicateEventsBadge(2)).toBeVisible();

      // Switch to dedicated Duplicates tab (1 candidate pair + 2 event groups = 3).
      await quality.openDuplicatesTab(3);

      const birthCard = quality.eventCard(BIRTH_SAFE_GROUP.id);
      await expect(birthCard).toBeVisible();
      await expect(
        birthCard.getByText(exact(quality.t('domain.events.birth')))
      ).toBeVisible();
      await expect(
        birthCard.getByText(
          exact(quality.t('quality.duplicateEvents.eventsCountBadge', { count: 2 }))
        )
      ).toBeVisible();
      await expect(
        birthCard.getByText(
          exact(quality.t('quality.duplicateEvents.readyToMergeBadge'))
        )
      ).toBeVisible();
      await expect(birthCard).toContainText(BIRTH_SAFE_GROUP.person.display_name);
      await expect(birthCard).toContainText('E1001');
      await expect(birthCard).toContainText('E1002');
      await expect(birthCard).toContainText('GAOO Fond 32 | Family bible note');
      await expect(
        birthCard.getByText(
          exact(
            quality.t('quality.duplicateEvents.combinedMediaCount', { count: 3 })
          )
        )
      ).toBeVisible();

      await expect(birthCard).toContainText(
        quality.t('quality.duplicateEvents.rationaleMatch', {
          count: 2,
          eventType: quality.t('domain.events.birth'),
          subject: BIRTH_SAFE_GROUP.person.display_name,
        })
      );
      await expect(
        birthCard.getByText(exact(quality.t('quality.duplicateEvents.conflictTag')))
      ).toHaveCount(0);
      await expect(quality.mergeButton(BIRTH_SAFE_GROUP.id)).toHaveClass(/bg-brand/);
      await expect(quality.dismissButton(BIRTH_SAFE_GROUP.id)).toHaveClass(/bg-surface/);

      const marriageCard = quality.eventCard(MARRIAGE_CONFLICT_GROUP.id);
      await expect(marriageCard).toBeVisible();
      await expect(
        marriageCard.getByText(exact(quality.t('domain.events.marriage')))
      ).toBeVisible();
      await expect(
        marriageCard.getByText(
          exact(quality.t('quality.duplicateEvents.reviewConflictBadge'))
        )
      ).toBeVisible();
      await expect(marriageCard).toContainText(
        MARRIAGE_CONFLICT_GROUP.person.display_name
      );
      await expect(marriageCard).toContainText(
        MARRIAGE_CONFLICT_GROUP.partner!.display_name
      );
      await expect(marriageCard).toContainText(
        quality.t('quality.duplicateEvents.rationaleConflict', {
          count: 2,
          eventType: quality.t('domain.events.marriage'),
          subject: `${MARRIAGE_CONFLICT_GROUP.person.display_name} & ${MARRIAGE_CONFLICT_GROUP.partner!.display_name}`,
          fields: `${quality.t('quality.duplicateEvents.fieldLabels.date')}, ${quality.t('quality.duplicateEvents.fieldLabels.place')}`,
        })
      );
      // UX-128: Conflict fields display Conflict tags and Customize becomes primary CTA.
      const conflictTags = marriageCard.getByText(
        exact(quality.t('quality.duplicateEvents.conflictTag'))
      );
      await expect(conflictTags.first()).toBeVisible();
      expect(await conflictTags.count()).toBeGreaterThanOrEqual(2);
      await expect(quality.customizeButton(MARRIAGE_CONFLICT_GROUP.id)).toHaveClass(
        /bg-brand/
      );
      await expect(quality.mergeButton(MARRIAGE_CONFLICT_GROUP.id)).toHaveClass(
        /bg-surface/
      );

      await quality.assertNoRawI18nLeaks();
    });

    test('1-click merge sends merge payload and Undo toast restores the group', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeVisible();
      await quality.mergeButton(BIRTH_SAFE_GROUP.id).click();

      const mergeReq = api.last('POST', '/quality/events/merge');
      expect(mergeReq?.body).toEqual({
        group_id: BIRTH_SAFE_GROUP.id,
        keep_event_id: 'evt-birth-1a',
        merge_event_ids: ['evt-birth-1b'],
        date_text: '1924-05-12',
        place_id: 'pl-samara',
        place_name: 'Samara, Samara Governorate',
        description: 'Church parish entry',
        source_text: 'GAOO Fond 32 | Family bible note',
      });

      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeHidden();
      await expect(quality.summaryDuplicateEventsBadge(1)).toBeVisible();
      await expect(quality.duplicatesTab(2)).toBeVisible();

      await expect(quality.undoToast).toBeVisible();
      await expect(quality.undoToast).toContainText(
        quality.t('quality.duplicateEvents.mergedToastTitle')
      );
      await quality.assertUndoToastNotClipped();

      await quality.undo();

      const undoReq = api.last('POST', '/history/tx/');
      expect(undoReq?.path).toBe('/history/tx/tx-qual-1/undo');

      await expect(
        quality.toast(quality.t('history.undo_confirmed_toast'))
      ).toBeVisible();
      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeVisible();
      await expect(quality.summaryDuplicateEventsBadge(2)).toBeVisible();
      await expect(quality.duplicatesTab(3)).toBeVisible();
    });

    test('customize modal switches surviving event and omits stale place_id on custom place', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await quality.openCustomizeModal(MARRIAGE_CONFLICT_GROUP.id);

      await expect(quality.keepEventRadio('evt-marr-1a')).toHaveAttribute(
        'aria-checked',
        'true'
      );

      // Select second recorded marriage event and pick quick-fill date choice.
      await quality.keepEventRadio('evt-marr-1b').click();
      await expect(quality.keepEventRadio('evt-marr-1b')).toHaveAttribute(
        'aria-checked',
        'true'
      );

      await quality.quickFillButton('1974-07-02').click();
      await expect(quality.dateInput).toHaveValue('1974-07-02');

      // Entering a custom place name must drop any stale place_id from variants.
      await quality.placeInput.fill('Saint Petersburg');
      await quality.confirmCustomizeButton.click();

      await expect(quality.dialog).toBeHidden();
      await expect(quality.eventCard(MARRIAGE_CONFLICT_GROUP.id)).toBeHidden();

      const mergeReq = api.last('POST', '/quality/events/merge');
      expect(mergeReq?.body).toEqual({
        group_id: MARRIAGE_CONFLICT_GROUP.id,
        keep_event_id: 'evt-marr-1b',
        merge_event_ids: ['evt-marr-1a'],
        date_text: '1974-07-02',
        place_name: 'Saint Petersburg',
        description: 'Civil registry',
        source_text: 'ZAGS #4 Moscow',
      });
      expect(mergeReq?.body).not.toHaveProperty('place_id');
    });

    test('customize modal sends explicit empty strings when clearing metadata fields', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await quality.openCustomizeModal(MARRIAGE_CONFLICT_GROUP.id);

      await quality.dateInput.fill('');
      await quality.placeInput.fill('');
      await quality.descriptionInput.fill('');
      await quality.sourceInput.fill('');
      await quality.confirmCustomizeButton.click();

      await expect(quality.dialog).toBeHidden();
      await expect(quality.eventCard(MARRIAGE_CONFLICT_GROUP.id)).toBeHidden();

      const mergeReq = api.last('POST', '/quality/events/merge');
      expect(mergeReq?.body).toEqual({
        group_id: MARRIAGE_CONFLICT_GROUP.id,
        keep_event_id: 'evt-marr-1a',
        merge_event_ids: ['evt-marr-1b'],
        date_text: '',
        place_name: '',
        description: '',
        source_text: '',
      });
      expect(mergeReq?.body).not.toHaveProperty('place_id');
    });

    test('batch auto-merge merges safe groups only, keeps conflicts, and undo restores all', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page, {
        includeSecondSafeGroup: true,
      });
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await expect(quality.allEventCards).toHaveCount(3);
      await expect(quality.batchMergeButton).toContainText(
        quality.t('quality.duplicateEvents.autoMergeAllBtn', { count: 2 })
      );

      await quality.batchMergeButton.click();

      const batchReq = api.last('POST', '/quality/events/batch-merge');
      expect(batchReq?.body).toEqual({
        group_ids: [BIRTH_SAFE_GROUP.id, DEATH_SAFE_GROUP.id],
        only_high_confidence: true,
      });

      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeHidden();
      await expect(quality.eventCard(DEATH_SAFE_GROUP.id)).toBeHidden();
      await expect(quality.eventCard(MARRIAGE_CONFLICT_GROUP.id)).toBeVisible();
      await expect(quality.batchMergeButton).toBeHidden();
      await expect(quality.summaryDuplicateEventsBadge(1)).toBeVisible();
      await expect(quality.duplicatesTab(2)).toBeVisible();

      await expect(quality.undoToast).toBeVisible();
      await expect(quality.undoToast).toContainText(
        quality.t('quality.duplicateEvents.batchMergedToastTitle', { count: 2 })
      );
      await quality.assertUndoToastNotClipped();

      await quality.undo();

      const undoReq = api.last('POST', '/history/tx/');
      expect(undoReq?.path).toBe('/history/tx/tx-qual-1/undo');

      await expect(quality.allEventCards).toHaveCount(3);
      await expect(quality.summaryDuplicateEventsBadge(3)).toBeVisible();
      await expect(quality.batchMergeButton).toBeVisible();
    });

    test('dismiss ("Looks Correct") dismisses warning and shows empty state when all cleared', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await quality.dismissButton(MARRIAGE_CONFLICT_GROUP.id).click();

      const firstDismiss = api.last('POST', '/quality/warnings/dismiss');
      expect(firstDismiss?.body).toMatchObject({
        rule_code: 'DUPLICATE_EVENT',
        entity_id: MARRIAGE_CONFLICT_GROUP.id,
      });
      await expect(quality.eventCard(MARRIAGE_CONFLICT_GROUP.id)).toBeHidden();
      await expect(
        quality.toast(quality.t('quality.duplicateEvents.dismissedToast'))
      ).toBeVisible();
      await expect(quality.summaryDuplicateEventsBadge(1)).toBeVisible();

      await quality.dismissButton(BIRTH_SAFE_GROUP.id).click();

      const secondDismiss = api.last('POST', '/quality/warnings/dismiss');
      expect(secondDismiss?.body).toMatchObject({
        rule_code: 'DUPLICATE_EVENT',
        entity_id: BIRTH_SAFE_GROUP.id,
      });
      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeHidden();
      await expect(quality.emptyEventsHeading).toBeVisible();
      await expect(quality.summaryDuplicateEventsBadge(0)).toBeVisible();
    });

    test('shows error toast without undo/success when merge returns 404 (DEF-003)', async ({
      page,
    }) => {
      await installQualityApiMock(page, {
        mergeError: {
          status: 404,
          code: 'event_not_found',
          message: 'Not found',
        },
      });
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await quality.mergeButton(BIRTH_SAFE_GROUP.id).click();

      await expect(quality.errorToast).toBeVisible();
      await expect(quality.undoToast).toHaveCount(0);
      await expect(quality.successToast).toHaveCount(0);
      await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeVisible();
      await expect(quality.summaryDuplicateEventsBadge(2)).toBeVisible();
    });

    test('navigates from warning Review Duplicates CTA to highlighted event card (UX-132)', async ({
      page,
    }) => {
      await installQualityApiMock(page, { includeDuplicateEventWarning: true });
      const quality = new QualityPage(page, locale);
      await quality.open('/quality');

      const reviewBtn = quality.warningReviewDupEventsButton(BIRTH_SAFE_GROUP.id);
      await expect(reviewBtn).toBeVisible();
      await expect(reviewBtn).toContainText(
        quality.t('quality.duplicateEvents.reviewDuplicatesBtn')
      );

      await reviewBtn.click();

      await expect(quality.duplicatesTab(3)).toHaveAttribute(
        'aria-checked',
        'true'
      );
      const targetCard = quality.eventCard(BIRTH_SAFE_GROUP.id);
      await expect(targetCard).toBeVisible();
      await expect(targetCard).toHaveClass(/ring-2/);
      await expect(targetCard).toBeFocused();
    });

    test('combines distinct archival sources via combine-sources-chip in customize modal (UX-134)', async ({
      page,
    }) => {
      const api = await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      await quality.openCustomizeModal(MARRIAGE_CONFLICT_GROUP.id);
      await expect(quality.combineSourcesChip).toBeVisible();
      await expect(quality.combineSourcesChip).toContainText(
        quality.t('quality.duplicateEvents.combineSourcesBtn')
      );

      await quality.combineSourcesChip.click();
      await expect(quality.sourceInput).toHaveValue(
        'ZAGS #4 Moscow | Parish book 1974'
      );

      await quality.confirmCustomizeButton.click();
      await expect(quality.dialog).toBeHidden();

      const mergeReq = api.last('POST', '/quality/events/merge');
      expect(mergeReq?.body).toMatchObject({
        group_id: MARRIAGE_CONFLICT_GROUP.id,
        source_text: 'ZAGS #4 Moscow | Parish book 1974',
      });
    });

    test('traps initial and keyboard Tab/Shift+Tab focus inside Customize modal (UX-127)', async ({
      page,
    }) => {
      await installQualityApiMock(page);
      const quality = new QualityPage(page, locale);
      await quality.open('/quality?tab=duplicates');

      const trigger = quality.customizeButton(MARRIAGE_CONFLICT_GROUP.id);
      await trigger.focus();
      await trigger.click();
      await expect(quality.dialog).toBeVisible();

      const isFocusInsideDialog = () =>
        page.evaluate(() => {
          const dlg = document.querySelector('[role="dialog"]');
          return Boolean(
            dlg && document.activeElement && dlg.contains(document.activeElement)
          );
        });

      await expect.poll(isFocusInsideDialog).toBe(true);

      const isBgButtonInert = await page.evaluate((id) => {
        const bg = document.querySelector(
          `[data-testid="merge-duplicate-event-btn-${id}"]`
        );
        return Boolean(bg?.closest('[inert]'));
      }, MARRIAGE_CONFLICT_GROUP.id);
      expect(isBgButtonInert).toBe(true);

      for (let i = 0; i < 20; i += 1) {
        await page.keyboard.press('Tab');
        expect(await isFocusInsideDialog()).toBe(true);
      }
      for (let i = 0; i < 5; i += 1) {
        await page.keyboard.press('Shift+Tab');
        expect(await isFocusInsideDialog()).toBe(true);
      }

      await page.keyboard.press('Escape');
      await expect(quality.dialog).toBeHidden();
      await expect(trigger).toBeFocused();
    });
  });
}

test.describe('Data Quality — Responsive & Live Localization', () => {
  test.beforeEach(async ({ setLocale, mockAuthenticatedState }) => {
    await setLocale('en');
    await mockAuthenticatedState({ role: 'admin' });
  });

  test('switches EN -> RU live and renders localized event types, badges, and customize modal cleanly', async ({
    page,
  }) => {
    await installQualityApiMock(page, { includeSecondSafeGroup: true });
    const quality = new QualityPage(page, 'en');
    await quality.open('/quality?tab=duplicates');

    await quality.switchLocaleLive('ru');

    await expect(quality.summaryDuplicateEventsBadge(3)).toBeVisible();
    await expect(
      quality.eventCard(BIRTH_SAFE_GROUP.id).getByText(exact('Рождение'))
    ).toBeVisible();
    await expect(
      quality
        .eventCard(MARRIAGE_CONFLICT_GROUP.id)
        .getByText(exact('Брак / Венчание'))
    ).toBeVisible();
    await expect(
      quality
        .eventCard(BIRTH_SAFE_GROUP.id)
        .getByText(exact('Готово к объединению'))
    ).toBeVisible();
    await expect(
      quality
        .eventCard(MARRIAGE_CONFLICT_GROUP.id)
        .getByText(exact('Требует проверки'))
    ).toBeVisible();

    await quality.assertNoRawI18nLeaks();

    await quality.openCustomizeModal(MARRIAGE_CONFLICT_GROUP.id);
    await expect(quality.dialog).toContainText('Настройка объединения событий');
    const modalText = await quality.dialog.innerText();
    expect(modalText).not.toMatch(/\b(quality|common)\.[a-zA-Z0-9_.-]+/);
    await quality.cancelCustomizeButton.click();
    await expect(quality.dialog).toBeHidden();
  });

  test('renders without horizontal overflow or clipped controls across viewports (EN & RU)', async ({
    page,
  }, testInfo) => {
    const isMobileProject = testInfo.project.name.includes('mobile');
    if (!isMobileProject) {
      await page.setViewportSize({ width: 1440, height: 900 });
    }

    await installQualityApiMock(page, { includeSecondSafeGroup: true });
    const quality = new QualityPage(page, 'en');
    await quality.open('/quality?tab=duplicates');

    const allGroupIds = [
      BIRTH_SAFE_GROUP.id,
      DEATH_SAFE_GROUP.id,
      MARRIAGE_CONFLICT_GROUP.id,
    ];
    await quality.assertFilterTabsSingleLineNoClip();
    await quality.assertResponsiveNoOverflow(allGroupIds);

    await quality.switchLocaleLive('ru');
    await quality.assertFilterTabsSingleLineNoClip();
    await quality.assertResponsiveNoOverflow(allGroupIds);
  });

  test('renders localized Appearance heading in mobile RU drawer without raw i18n keys (UX-136)', async ({
    page,
    setLocale,
  }, testInfo) => {
    if (!testInfo.project.name.includes('mobile')) {
      await page.setViewportSize({ width: 412, height: 915 });
    }
    await setLocale('ru');
    await installQualityApiMock(page);

    const quality = new QualityPage(page, 'ru');
    await quality.open('/quality?tab=duplicates');

    await expect(quality.mobileMenuButton.first()).toBeVisible();
    await quality.mobileMenuButton.first().click();

    await expect(quality.mobileNavDrawer).toBeVisible();
    await expect(
      quality.mobileNavDrawer.getByText(exact('Оформление'))
    ).toBeVisible();

    const drawerText = await quality.mobileNavDrawer.innerText();
    expect(drawerText).not.toContain('settings.tabs.appearance');
    expect(drawerText).not.toMatch(/\b(settings|nav|common)\.[a-zA-Z0-9_.-]+/);

    await page.keyboard.press('Escape');
    await expect(quality.mobileNavDrawer).toBeHidden();
  });

  test('renders localized empty state when API returns zero duplicate events', async ({
    page,
  }) => {
    await installQualityApiMock(page, { emptyEvents: true });
    const quality = new QualityPage(page, 'en');
    await quality.open('/quality?tab=duplicates');

    await expect(quality.emptyEventsHeading).toBeVisible();
    await expect(quality.batchMergeButton).toBeHidden();
    await expect(quality.summaryDuplicateEventsBadge(0)).toBeVisible();
  });

  test('non-admin viewer and editor see AccessDeniedCard and cannot inspect duplicate events', async ({
    page,
    mockAuthenticatedState,
  }) => {
    for (const role of ['viewer', 'editor'] as const) {
      await mockAuthenticatedState({ role });
      await installQualityApiMock(page);
      const quality = new QualityPage(page, 'en');
      await quality.goto('/quality?tab=duplicates');
      await expect(
        page.getByRole('heading', { name: 'Administrator Access Required' })
      ).toBeVisible();
      await expect(quality.duplicateEventsSection).toHaveCount(0);
    }
  });

  test('toggles dark theme, opens/closes Customize modal via keyboard with visible focus ring, and handles 500 merge errors', async ({
    page,
  }) => {
    await installQualityApiMock(page);
    const quality = new QualityPage(page, 'en');
    await quality.open('/quality?tab=duplicates');

    for (let i = 0; i < 3; i += 1) {
      if (await quality.isDarkMode()) break;
      await quality.toggleTheme();
    }
    const mobileDrawer = page.locator('aside[role="dialog"]');
    if (await mobileDrawer.isVisible()) {
      await page.keyboard.press('Escape');
      await expect(mobileDrawer).toBeHidden();
    }
    await expect
      .poll(() => quality.isDarkMode())
      .toBe(true);

    const customizeBtn = quality.customizeButton(BIRTH_SAFE_GROUP.id);
    await customizeBtn.focus();
    await expect(customizeBtn).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(quality.dialog).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(quality.dialog).toBeHidden();

    // 500 error keeps the card intact and displays an error toast
    await page.route('**/api/v1/quality/events/merge', async (route) => {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          error: { code: 'internal_error', message: 'Database unavailable' },
        }),
      });
    });

    await quality.mergeButton(BIRTH_SAFE_GROUP.id).click();
    await expect(quality.eventCard(BIRTH_SAFE_GROUP.id)).toBeVisible();
    await expect(page.getByTestId('toast-error')).toBeVisible();
  });
});
