import { test, expect } from './fixtures/test-base';
import { tr, type Locale } from './fixtures/i18n';
import {
  HISTORY_SUBJECT_ID,
  HISTORY_TX_IDS,
  installHistoryApiMock,
} from './fixtures/history-api-mock';
import {
  HistoryVisualizationPage,
  resolveIcuTemplate,
} from './pages/history.page';

const LOCALES: readonly Locale[] = ['en', 'ru'];

test.describe('Change History Visualization & Localization', () => {
  for (const locale of LOCALES) {
    test.describe(`[${locale.toUpperCase()}] /history audit feed`, () => {
      test('renders localized tx labels, deduplicated entity GIDs, fields, values, and zero UUID/JSON leaks', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        await installHistoryApiMock(page);

        const hp = new HistoryVisualizationPage(page);
        await hp.openHistoryPage(locale);

        // 1. Localized composite transaction labels (never raw "Person.update Profile")
        const profileCard = hp.txCard(HISTORY_TX_IDS.profileUpdate);
        await expect(hp.txLabel(HISTORY_TX_IDS.profileUpdate)).toHaveText(
          `${tr(locale, 'history.txLabels.personUpdateProfile')} — Alexander Levshits`
        );

        const familyCard = hp.txCard(HISTORY_TX_IDS.familyUpdate);
        await expect(hp.txLabel(HISTORY_TX_IDS.familyUpdate)).toHaveText(
          `${tr(locale, 'history.txLabels.familyUpdate')} — Alexander Levshits & Vera Levshits`
        );

        // 2. Deduplicated entity_label + separate monospace entity_gid pill inside entry row title (no "(F0100)" duplicated in title)
        const familyEntryRow = familyCard.locator('div.bg-canvas').first();
        const familyEntryTitle = familyEntryRow.locator('.font-medium').first();
        await expect(familyEntryTitle).toHaveText(
          `Alexander Levshits & Vera Levshits (${tr(locale, 'history.entityTypes.family')})`
        );
        await expect(familyEntryTitle).not.toContainText('(F0100)');
        await expect(
          familyEntryRow.locator('span.font-mono', { hasText: /^F0100$/ })
        ).toBeVisible();

        const profileEntryRow = profileCard.locator('div.bg-canvas').first();
        const profileEntryTitle = profileEntryRow.locator('.font-medium').first();
        await expect(profileEntryTitle).not.toContainText('(I0001)');
        await expect(
          profileEntryRow.locator('span.font-mono', { hasText: /^I0001$/ })
        ).toBeVisible();

        const eventMergeCard = hp.txCard(HISTORY_TX_IDS.eventMerge);
        const eventEntryRow = eventMergeCard.locator('div.bg-canvas').first();
        const eventEntryTitle = eventEntryRow.locator('.font-medium').first();
        await expect(eventEntryTitle).not.toContainText('(E0105)');
        await expect(
          eventEntryRow.locator('span.font-mono', { hasText: /^E0105$/ })
        ).toBeVisible();

        // 3. Localized field labels (never raw snake_case keys)
        const expectedFieldKeys = [
          'birth_date',
          'sex',
          'partner1_id',
          'children',
          'merged_profile',
          'merged_events',
          'undone_tx_id',
          'record_counts',
          'restored_snapshot_id',
          'safety_snapshot_id',
          'restored_entity_ids',
          'face_tags',
        ] as const;
        for (const fieldKey of expectedFieldKeys) {
          await expect(hp.pageRoot).toContainText(
            tr(locale, `history.fields.${fieldKey}`)
          );
          await expect(hp.pageRoot).not.toContainText(fieldKey);
        }

        // 4. Human-readable diff values
        // GenDate + localized sex
        await expect(profileCard).toContainText(
          locale === 'ru' ? 'около 1912 г.' : 'about 1912'
        );
        await expect(profileCard).toContainText(
          locale === 'ru' ? '19 марта 1912 г.' : '19 Mar 1912'
        );
        await expect(profileCard).toContainText(tr(locale, 'domain.sex.unknown'));
        await expect(profileCard).toContainText(tr(locale, 'domain.sex.male'));

        // Family child (adopted) badge localized + FamilyChildDto object localized without UUID leak
        const localizedAdopted = tr(locale, 'domain.pedigree.adopted');
        await expect(familyCard).toContainText(`Elena Levshits (I0042) (${localizedAdopted})`);
        await expect(familyCard).toContainText(`Viktor Levshits (I0043) (${localizedAdopted})`);

        // Media face_tags [primary avatar] localized
        const mediaCard = hp.txCard(HISTORY_TX_IDS.mediaTags);
        const localizedPrimaryPhoto = tr(
          locale,
          'history.fields.is_primary_avatar'
        ).toLowerCase();
        await expect(mediaCard).toContainText(
          `Pyotr Kalman (I0008) [${localizedPrimaryPhoto}]`
        );

        // Person merge bundle & event merge bundle
        const personMergeCard = hp.txCard(HISTORY_TX_IDS.personMerge);
        await expect(personMergeCard).toContainText('Sasha Levshits (I0099)');
        await expect(personMergeCard).toContainText('Alexander Levshits (I0001)');

        const birthWord = tr(locale, 'domain.events.birth');
        await expect(eventMergeCard).toContainText(
          `${birthWord} — Maria Petrova (E0104)`
        );
        await expect(eventMergeCard).toContainText(
          `${birthWord} — Maria Petrova (E0105)`
        );

        // Undo + Rollback metadata, "Note — <target>", "birth — <owner>", and ICU record_counts
        const rollbackCard = hp.txCard(HISTORY_TX_IDS.undoAndRollback);
        await expect(rollbackCard).toContainText(
          `${tr(locale, 'history.txLabels.personUpdateProfile')} — Alexander Levshits`
        );
        await expect(rollbackCard).toContainText(
          `${tr(locale, 'history.entityTypes.note')} — Alexander Levshits`
        );
        await expect(rollbackCard).toContainText(
          `${birthWord} — Maria Petrova`
        );

        const expectedCounts = [
          resolveIcuTemplate(tr(locale, 'history.recordCounts.individuals'), {
            count: 5,
          }),
          resolveIcuTemplate(tr(locale, 'history.recordCounts.families'), {
            count: 2,
          }),
          resolveIcuTemplate(tr(locale, 'history.recordCounts.events'), {
            count: 1,
          }),
          resolveIcuTemplate(tr(locale, 'history.recordCounts.places'), {
            count: 4,
          }),
        ];
        for (const chunk of expectedCounts) {
          await expect(rollbackCard).toContainText(chunk);
        }

        // DeletedPersonSnapshotPayload unwrapped without leaking families_before or UUIDs
        const deleteCard = hp.txCard(HISTORY_TX_IDS.deletePerson);
        await expect(deleteCard).toContainText('Nikolai');
        await expect(deleteCard).toContainText('Orlov');
        await expect(deleteCard).not.toContainText('families_before');

        if (locale === 'ru') {
          const fullText = await hp.pageRoot.innerText();
          expect(fullText).not.toContain('[adopted]');
          expect(fullText).not.toContain('[primary avatar]');
          expect(fullText).not.toContain('Note — Alexander Levshits');
          expect(fullText).not.toContain('birth — Maria Petrova');
        }

        // 5. Strict zero-UUID, zero-raw-JSON, zero-raw-i18n checks
        await hp.assertNoUuidOrJsonLeaks(hp.pageRoot);
        await hp.assertNoRawI18nLeaks(hp.pageRoot);
      });

      test('propagates backend errors as error toasts on Undo & Restore and succeeds when backend succeeds', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        const apiMock = await installHistoryApiMock(page, {
          undoError: {
            status: 500,
            code: 'internal_error',
            message: 'Simulated undo failure',
          },
          restoreError: {
            status: 404,
            code: 'not_found',
            message: 'Revision no longer exists',
          },
        });

        const hp = new HistoryVisualizationPage(page);
        await hp.openHistoryPage(locale);

        // Undo 500 -> error toast must show dedicated undoErrorTitle + localized error, NEVER a false success toast
        await hp.undoButton(HISTORY_TX_IDS.profileUpdate).click();
        await expect(hp.errorToast).toHaveCount(1);
        await expect(hp.errorToast.first()).toContainText(
          tr(locale, 'history.undoErrorTitle')
        );
        await expect(hp.errorToast.first()).toContainText(
          tr(locale, 'errors.internal_error')
        );
        await expect(hp.successToast).toHaveCount(0);
        expect(apiMock.find('POST', '/undo')).toHaveLength(1);

        // Trigger restore 404 -> stacked second error toast must show dedicated restore.errorTitle + not_found
        await hp.restoreDeletedButton(108).click();
        await expect(hp.errorToast).toHaveCount(2);
        await expect(hp.errorToast.last()).toContainText(
          tr(locale, 'history.restore.errorTitle')
        );
        await expect(hp.errorToast.last()).toContainText(
          tr(locale, 'errors.not_found')
        );
        await expect(hp.successToast).toHaveCount(0);
        expect(apiMock.find('POST', '/restore')).toHaveLength(1);
      });

      test('happy-path transaction Undo and Restore Deleted Record update feed and show localized confirmation toasts', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        const apiMock = await installHistoryApiMock(page);

        const hp = new HistoryVisualizationPage(page);
        await hp.openHistoryPage(locale);

        const undoBtn = hp.undoButton(HISTORY_TX_IDS.profileUpdate);
        await expect(undoBtn).toBeVisible();
        await undoBtn.click();

        await expect(hp.successToast).toBeVisible();
        await expect(hp.successToast).toContainText(
          tr(locale, 'history.undo_confirmed_toast')
        );
        await expect(undoBtn).toBeHidden();
        expect(apiMock.find('POST', '/undo')).toHaveLength(1);

        // Restore deleted person entry from /history feed
        const restoreBtn = hp.restoreDeletedButton(108);
        await expect(restoreBtn).toBeVisible();
        await restoreBtn.click();
        await expect(
          hp.successToast.filter({
            hasText: tr(locale, 'history.restore.successToast'),
          })
        ).toBeVisible();
        expect(apiMock.find('POST', '/history/108/restore')).toHaveLength(1);
      });
    });

    test.describe(`[${locale.toUpperCase()}] RecordHistoryDrawer on person profile`, () => {
      test('shows localized diffs, hides Restore button on create and non-revertible entries, and handles restore + errors', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        const apiMock = await installHistoryApiMock(page);

        const hp = new HistoryVisualizationPage(page);
        await hp.openPersonProfile(HISTORY_SUBJECT_ID, locale);
        await hp.openRecordHistoryDrawer(locale);

        const drawerDialog = hp.recordHistoryDrawerDialog(locale);
        await expect(drawerDialog).toContainText('Alexander Sergeyevich Levshits');

        // Entry 901: can_revert: true -> auto-expanded and shows Restore button
        const revertibleEntry = hp.drawerEntry(901);
        await expect(revertibleEntry).toBeVisible();
        await expect(revertibleEntry).toContainText(
          tr(locale, 'history.fields.birth_date')
        );
        await expect(revertibleEntry).toContainText(
          locale === 'ru' ? '19 марта 1912 г.' : '19 Mar 1912'
        );
        await expect(revertibleEntry).toContainText(tr(locale, 'domain.sex.male'));
        const restoreBtn = revertibleEntry.getByRole('button', {
          name: tr(locale, 'history.drawer.restoreVersionBtn'),
        });
        await expect(restoreBtn).toBeVisible();

        // Entry 902: update with can_revert: false -> Restore button MUST be hidden
        const nonRevertibleUpdate = hp.drawerEntry(902);
        await expect(nonRevertibleUpdate).toBeVisible();
        await nonRevertibleUpdate
          .getByRole('button', { name: /field changes|изменённые поля/i })
          .click();
        await expect(nonRevertibleUpdate).toContainText(
          tr(locale, 'history.fields.face_tags')
        );
        await expect(
          nonRevertibleUpdate.getByRole('button', {
            name: tr(locale, 'history.drawer.restoreVersionBtn'),
          })
        ).toHaveCount(0);

        // Entry 903: op: 'create' -> Restore button MUST be hidden
        const createEntry = hp.drawerEntry(903);
        await expect(createEntry).toBeVisible();
        await expect(
          createEntry.getByRole('button', {
            name: tr(locale, 'history.drawer.restoreVersionBtn'),
          })
        ).toHaveCount(0);

        await hp.assertNoUuidOrJsonLeaks(hp.drawerContainer);
        await hp.assertNoRawI18nLeaks(hp.drawerContainer);

        // Happy-path restore confirmation
        await restoreBtn.click();
        const confirmDialog = hp.restoreConfirmDialog(locale);
        await expect(confirmDialog).toBeVisible();
        await confirmDialog
          .getByRole('button', { name: tr(locale, 'history.restore.confirmBtn') })
          .click();

        await expect(hp.successToast).toContainText(
          tr(locale, 'history.restore.successToast')
        );
        expect(apiMock.find('POST', '/history/901/restore')).toHaveLength(1);
      });

      test('surfaces localized error alert when RecordHistoryDrawer restore fails with HTTP 500', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        await installHistoryApiMock(page, {
          restoreError: {
            status: 500,
            code: 'internal_error',
            message: 'Restore transaction aborted',
          },
        });

        const hp = new HistoryVisualizationPage(page);
        await hp.openPersonProfile(HISTORY_SUBJECT_ID, locale);
        await hp.openRecordHistoryDrawer(locale);

        await hp
          .drawerEntry(901)
          .getByRole('button', {
            name: tr(locale, 'history.drawer.restoreVersionBtn'),
          })
          .click();
        const confirmDialog = hp.restoreConfirmDialog(locale);
        await confirmDialog
          .getByRole('button', { name: tr(locale, 'history.restore.confirmBtn') })
          .click();

        await expect(confirmDialog.getByRole('alert')).toBeVisible();
        await expect(confirmDialog.getByRole('alert')).toContainText(
          tr(locale, 'errors.internal_error')
        );
        await expect(hp.successToast).toHaveCount(0);
      });
    });

    test.describe(`[${locale.toUpperCase()}] ConflictResolutionModal (409 Optimistic Concurrency)`, () => {
      test('renders localized fields, localized sex values, formatted timestamp (never bare "2"), and retries PATCH with remote version', async ({
        page,
        setLocale,
        mockAuthenticatedState,
      }) => {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Lev Levshits' });
        const apiMock = await installHistoryApiMock(page, {
          conflictOnStalePatch: true,
          remoteVersion: 3,
        });

        const hp = new HistoryVisualizationPage(page);
        await hp.openPersonProfile(HISTORY_SUBJECT_ID, locale);

        const editDialog = await hp.openPersonEditDialog(locale);
        await editDialog
          .getByLabel(tr(locale, 'person.edit.surnameLabel'), { exact: true })
          .fill('Levshits-Kazan');
        await editDialog
          .getByLabel(tr(locale, 'person.edit.biographyLabel'), { exact: true })
          .fill('Edited locally in current tab.');

        await editDialog
          .getByRole('button', { name: tr(locale, 'person.edit.saveBtn') })
          .click();

        const conflictDialog = hp.conflictDialog(locale);
        await expect(conflictDialog).toBeVisible();
        await expect(hp.conflictModalContent).toBeVisible();

        // Banner shows Vera Levshits + v3 badge + formatted timestamp from updated_at, never bare "(2)" / "(3)"
        await expect(hp.conflictModalContent).toContainText('Vera Levshits');
        await expect(hp.conflictModalContent).toContainText('v3');
        await expect(hp.conflictModalContent).toContainText('2026');
        const expectedMonthFragment = locale === 'ru' ? 'нояб' : 'Nov';
        await expect(hp.conflictModalContent).toContainText(expectedMonthFragment);
        await expect(hp.conflictModalContent).not.toContainText('(2)');
        await expect(hp.conflictModalContent).not.toContainText('(3)');

        // Localized field names and localized Sex values (Male vs Female)
        const surnameRow = hp.conflictFieldRow('surname');
        await expect(surnameRow).toContainText(tr(locale, 'history.fields.surname'));
        await expect(surnameRow).toContainText('Levshits-Kazan');
        await expect(surnameRow).toContainText('Levshits-Orlov');

        const sexRow = hp.conflictFieldRow('sex');
        await expect(sexRow).toContainText(tr(locale, 'history.fields.sex'));
        await expect(sexRow).toContainText(tr(locale, 'domain.sex.male'));
        await expect(sexRow).toContainText(tr(locale, 'domain.sex.female'));

        const bioRow = hp.conflictFieldRow('biography');
        await expect(bioRow).toContainText(tr(locale, 'history.fields.biography'));
        await expect(bioRow).toContainText('Edited locally in current tab.');
        await expect(bioRow).toContainText(
          'Saved remotely by Vera from archive census.'
        );

        await hp.assertNoUuidOrJsonLeaks(hp.conflictModalContent);
        await hp.assertNoRawI18nLeaks(hp.conflictModalContent);

        // Exercise "Merge field by field" -> choose remote surname -> Apply merge
        await hp.conflictModalContent
          .getByRole('button', {
            name: tr(locale, 'history.conflict.mergeFieldByFieldBtn'),
          })
          .click();

        await expect(
          hp.conflictModalContent.getByText(
            tr(locale, 'history.conflict.mergeInstruction')
          )
        ).toBeVisible();

        await surnameRow
          .getByRole('radio', { name: /Levshits-Orlov/i })
          .check();

        await hp.conflictModalContent
          .getByRole('button', {
            name: tr(locale, 'history.conflict.applyMergedBtn'),
          })
          .click();

        // Second PATCH must carry remoteVersion (3) and chosen surname, then close modal cleanly
        await expect
          .poll(() => apiMock.find('PATCH', `/persons/${HISTORY_SUBJECT_ID}`).length)
          .toBe(2);
        const patches = apiMock.find('PATCH', `/persons/${HISTORY_SUBJECT_ID}`);
        expect(patches[0].body?.version).toBe(1);
        expect(patches[1].body?.version).toBe(3);
        expect(patches[1].body?.surname).toBe('Levshits-Orlov');
        await expect(conflictDialog).toBeHidden();
      });
    });
  }

  test.describe('Responsive Viewports, Filtering, Live Locale Switch & RBAC', () => {
    test('renders cleanly with zero horizontal overflow at desktop (1280x800) and mobile (390x844) in EN and RU', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      for (const locale of LOCALES) {
        for (const viewport of [
          { width: 1280, height: 800 },
          { width: 390, height: 844 },
        ]) {
          await page.setViewportSize(viewport);
          await setLocale(locale);
          await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
          await installHistoryApiMock(page);

          const hp = new HistoryVisualizationPage(page);
          await hp.openHistoryPage(locale);
          await expect(hp.txCard(HISTORY_TX_IDS.profileUpdate)).toBeVisible();
          await expect(hp.txCard(HISTORY_TX_IDS.familyUpdate)).toBeVisible();
          await hp.assertNoHorizontalOverflow();
        }
      }
    });

    test('filters history entries by entity type & only-deletions and toggles EN <-> RU live', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      await setLocale('en');
      await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
      await installHistoryApiMock(page);

      const hp = new HistoryVisualizationPage(page);
      await hp.openHistoryPage('en');

      // Filter by Family
      await hp.selectFilterOption(
        hp.entityTypeCombobox('en'),
        tr('en', 'history.filters.entityFamily')
      );
      await expect(hp.txCard(HISTORY_TX_IDS.familyUpdate)).toBeVisible();
      await expect(hp.txCard(HISTORY_TX_IDS.profileUpdate)).toBeHidden();

      // Reset to All and toggle Only deletions
      await hp.selectFilterOption(
        hp.entityTypeCombobox('en'),
        tr('en', 'history.filters.entityAll')
      );
      await hp.onlyDeletionsSwitch('en').click();
      await expect(hp.txCard(HISTORY_TX_IDS.deletePerson)).toBeVisible();
      await expect(hp.txCard(HISTORY_TX_IDS.familyUpdate)).toBeHidden();

      // Turn off Only deletions, switch EN -> RU live via UI toggle
      await hp.onlyDeletionsSwitch('en').click();
      await expect(hp.txCard(HISTORY_TX_IDS.profileUpdate)).toBeVisible();
      await hp.toggleLanguage();
      const openNavDrawer = page.getByRole('dialog').first();
      if (await openNavDrawer.isVisible()) {
        await page.keyboard.press('Escape');
        await expect(openNavDrawer).toBeHidden();
      }

      await expect(
        page.getByRole('heading', { name: tr('ru', 'history.page.title') })
      ).toBeVisible();
      await expect(hp.txLabel(HISTORY_TX_IDS.profileUpdate)).toHaveText(
        `${tr('ru', 'history.txLabels.personUpdateProfile')} — Alexander Levshits`
      );
      await hp.assertNoRawI18nLeaks(hp.pageRoot);
    });

    test('non-admin viewer/editor receives AccessDeniedCard on /history', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      for (const role of ['viewer', 'editor'] as const) {
        await setLocale('en');
        await mockAuthenticatedState({ role, name: 'Family Guest' });
        await installHistoryApiMock(page);

        const hp = new HistoryVisualizationPage(page);
        await hp.goto('/history');

        await expect(
          page.getByRole('heading', { name: tr('en', 'rbac.accessDeniedTitle') })
        ).toBeVisible();
        await expect(hp.pageRoot).toHaveCount(0);
      }
    });
  });

  test.describe('Verified Concurrency Resolution, Navigation & Editor Accessibility', () => {
    test('ConflictResolutionModal displays remoteVersion (v3) and closes cleanly after Keep my changes', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      await setLocale('en');
      await mockAuthenticatedState({ role: 'admin', name: 'Lev Levshits' });
      const apiMock = await installHistoryApiMock(page, {
        conflictOnStalePatch: true,
        remoteVersion: 3,
      });

      const hp = new HistoryVisualizationPage(page);
      await hp.openPersonProfile(HISTORY_SUBJECT_ID, 'en');
      const editDialog = await hp.openPersonEditDialog('en');
      await editDialog
        .getByLabel(tr('en', 'person.edit.surnameLabel'), { exact: true })
        .fill('Levshits-Kazan');
      await editDialog
        .getByRole('button', { name: tr('en', 'person.edit.saveBtn') })
        .click();

      const conflictDialog = hp.conflictDialog('en');
      await expect(conflictDialog).toBeVisible();
      await expect(conflictDialog).toContainText('v3');

      await hp.conflictModalContent
        .getByRole('button', { name: tr('en', 'history.conflict.keepMineBtn') })
        .click();

      await expect
        .poll(() => apiMock.find('PATCH', `/persons/${HISTORY_SUBJECT_ID}`).length)
        .toBe(2);
      expect(
        apiMock.find('PATCH', `/persons/${HISTORY_SUBJECT_ID}`)[1].body?.version
      ).toBe(3);
      await expect(conflictDialog).toBeHidden();
      await expect(editDialog).toBeHidden();
    });

    test('clicking person entity link in /history navigates to /persons/:id and renders profile', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      await setLocale('en');
      await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
      await installHistoryApiMock(page);

      const hp = new HistoryVisualizationPage(page);
      await hp.openHistoryPage('en');

      await hp
        .txCard(HISTORY_TX_IDS.profileUpdate)
        .locator('div.bg-canvas')
        .first()
        .getByRole('button')
        .first()
        .click();

      await expect(page).toHaveURL(new RegExp(`/persons/${HISTORY_SUBJECT_ID}$`));
      await expect(
        page.getByRole('region', { name: tr('en', 'profile.hero.ariaLabel') })
      ).toBeVisible();
    });

    test('ProfileHeroHeader overflow trigger is an accessible button and Editor (can_edit: true, can_merge: false) can open Change History', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      await setLocale('en');
      await mockAuthenticatedState({ role: 'editor', name: 'Family Editor' });
      await installHistoryApiMock(page, { canEdit: true, canMerge: false });

      const hp = new HistoryVisualizationPage(page);
      await hp.openPersonProfile(HISTORY_SUBJECT_ID, 'en');

      const hero = page.getByRole('region', {
        name: tr('en', 'profile.hero.ariaLabel'),
      });
      const overflowButton = hero.getByRole('button', {
        name: tr('en', 'profile.actions.moreAria'),
      });
      await expect(overflowButton).toBeVisible();
      await overflowButton.click();

      const menu = page.getByRole('menu', {
        name: tr('en', 'profile.actions.moreAria'),
      });
      await expect(menu).toBeVisible();
      await expect(
        menu.getByRole('menuitem', { name: tr('en', 'profile.actions.mergeWith') })
      ).toHaveCount(0);

      await menu
        .getByRole('menuitem', { name: tr('en', 'profile.actions.viewHistory') })
        .click();
      await expect(hp.drawerContainer).toBeVisible();
    });
  });

  test.describe('Empty States, Date Bounds & Conflict Retry Resilience', () => {
    test('renders empty state on /history for empty 200 OK response and out-of-range fromDate filter without demo fallback rows', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      // 1. Direct empty 200 OK { transactions: [], items: [], total: 0 }
      await setLocale('en');
      await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
      await installHistoryApiMock(page, { emptyFeed: true });

      const hp = new HistoryVisualizationPage(page);
      await hp.openHistoryPage('en');

      await expect(
        hp.pageRoot.getByText(tr('en', 'history.page.emptyTitle'))
      ).toBeVisible();
      await expect(
        hp.pageRoot.getByText(tr('en', 'history.page.emptyDesc'))
      ).toBeVisible();
      await expect(page.locator('[data-testid^="history-tx-"]')).toHaveCount(0);

      // 2. Populated feed filtered down to 0 matches via fromDate
      await page.unrouteAll({ behavior: 'wait' });
      await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
      await installHistoryApiMock(page);
      await hp.openHistoryPage('en');
      await expect(hp.txCard(HISTORY_TX_IDS.profileUpdate)).toBeVisible();

      await hp.ensureFiltersExpanded();
      await hp.fromDateInput('en').fill('2099-01-01');
      await expect(page.locator('[data-testid^="history-tx-"]')).toHaveCount(0);
      await expect(
        hp.pageRoot.getByText(tr('en', 'history.page.emptyTitle'))
      ).toBeVisible();
    });

    test('renders empty state in RecordHistoryDrawer for empty 200 OK items without demo fallback rows', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      for (const locale of LOCALES) {
        await setLocale(locale);
        await mockAuthenticatedState({ role: 'admin', name: 'Vera Levshits' });
        await installHistoryApiMock(page, { emptyEntityHistory: true });

        const hp = new HistoryVisualizationPage(page);
        await hp.openPersonProfile(HISTORY_SUBJECT_ID, locale);
        await hp.openRecordHistoryDrawer(locale);

        await expect(
          hp.drawerContainer.getByText(tr(locale, 'history.drawer.emptyTitle'))
        ).toBeVisible();
        await expect(
          hp.drawerContainer.getByText(tr(locale, 'history.drawer.emptyDesc'))
        ).toBeVisible();
        await expect(
          hp.drawerContainer.locator('[data-testid^="history-entry-"]')
        ).toHaveCount(0);
      }
    });

    test('ConflictResolutionModal surfaces retry failure in conflict-error-alert and resets mergeMode on reopen', async ({
      page,
      setLocale,
      mockAuthenticatedState,
    }) => {
      await setLocale('en');
      await mockAuthenticatedState({ role: 'admin', name: 'Lev Levshits' });
      const apiMock = await installHistoryApiMock(page, {
        conflictOnStalePatch: true,
        remoteVersion: 3,
        patchRetryError: {
          status: 500,
          code: 'internal_error',
          message: 'Retry update failed',
        },
      });

      const hp = new HistoryVisualizationPage(page);
      await hp.openPersonProfile(HISTORY_SUBJECT_ID, 'en');
      const editDialog = await hp.openPersonEditDialog('en');

      await editDialog
        .getByLabel(tr('en', 'person.edit.surnameLabel'), { exact: true })
        .fill('Levshits-Kazan');
      await editDialog
        .getByRole('button', { name: tr('en', 'person.edit.saveBtn') })
        .click();

      const conflictDialog = hp.conflictDialog('en');
      await expect(conflictDialog).toBeVisible();

      // Switch to field-by-field merge mode and submit retry expecting HTTP 500
      await hp.conflictModalContent
        .getByRole('button', {
          name: tr('en', 'history.conflict.mergeFieldByFieldBtn'),
        })
        .click();
      await expect(
        hp.conflictModalContent.getByText(
          tr('en', 'history.conflict.mergeInstruction')
        )
      ).toBeVisible();

      await hp.conflictModalContent
        .getByRole('button', { name: tr('en', 'history.conflict.applyMergedBtn') })
        .click();

      await expect(hp.conflictErrorAlert()).toBeVisible();
      await expect(hp.conflictErrorAlert()).toContainText(
        tr('en', 'errors.internal_error')
      );
      await expect(conflictDialog).toBeVisible();
      expect(apiMock.find('PATCH', `/persons/${HISTORY_SUBJECT_ID}`)).toHaveLength(2);

      // Dismiss ConflictResolutionModal and reopen it by saving Edit Dialog again:
      // mergeMode and previous submit error must both be reset.
      await conflictDialog
        .getByRole('button', { name: tr('en', 'common.close') })
        .click();
      await expect(conflictDialog).toBeHidden();

      await editDialog
        .getByRole('button', { name: tr('en', 'person.edit.saveBtn') })
        .click();

      await expect(conflictDialog).toBeVisible();
      await expect(
        hp.conflictModalContent.getByRole('button', {
          name: tr('en', 'history.conflict.mergeFieldByFieldBtn'),
        })
      ).toBeVisible();
      await expect(
        hp.conflictModalContent.getByText(
          tr('en', 'history.conflict.mergeInstruction')
        )
      ).toHaveCount(0);
    });
  });
});
