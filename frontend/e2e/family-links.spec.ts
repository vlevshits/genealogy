import { test, expect } from './fixtures/test-base';
import { Locale } from './fixtures/i18n';
import {
  BORIS,
  CHILD,
  DMITRI,
  FATHER,
  GALINA,
  MOTHER,
  SPOUSE,
  SUBJECT,
  displayName,
  installFamilyApiMock,
} from './fixtures/family-api-mock';
import { PersonProfilePage } from './pages/person-profile.page';

const LOCALES: Locale[] = ['en', 'ru'];

for (const locale of LOCALES) {
  test.describe(`Profile family links (${locale})`, () => {
    test.beforeEach(async ({ setLocale }) => {
      await setLocale(locale);
    });

    test('unlink spouse then undo restores the spouse', async ({ page, mockAuthenticatedState }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await expect(profile.person(displayName(SPOUSE))).toBeVisible();
      await profile.unlinkSpouseButton.click();
      await expect(profile.dialog).toContainText(profile.t('profile.family.removeSpouseTitle'));
      await profile.confirmRemoval();

      await expect(profile.toast(profile.t('profile.family.spouseRemovedToast'))).toBeVisible();
      await expect(profile.person(displayName(SPOUSE))).toHaveCount(0);
      expect(api.find('DELETE', `/partners/${SPOUSE.id}`)).toHaveLength(1);

      await profile.undo();
      await expect(profile.person(displayName(SPOUSE))).toBeVisible();
      expect(api.find('POST', '/undo')).toHaveLength(1);
    });

    test('unlink child then undo restores the child', async ({ page, mockAuthenticatedState }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await expect(profile.person(displayName(CHILD))).toBeVisible();
      await profile.unlinkChildButton.click();
      await expect(profile.dialog).toContainText(profile.t('profile.family.removeChildTitle'));
      await profile.confirmRemoval();

      await expect(profile.toast(profile.t('profile.family.childRemovedToast'))).toBeVisible();
      await expect(profile.person(displayName(CHILD))).toHaveCount(0);
      expect(api.find('DELETE', `/children/${CHILD.id}`)).toHaveLength(1);

      await profile.undo();
      await expect(profile.person(displayName(CHILD))).toBeVisible();
    });

    test('two-parent family: "unlink only" keeps the other parent, undo restores both', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      // Father is rendered first.
      await profile.unlinkParentButtons.first().click();
      await expect(profile.dialog).toContainText(
        profile.t('profile.family.unlinkParentChoiceTitle', { name: displayName(FATHER) })
      );
      await expect(profile.dialog).toContainText(
        profile.t('profile.family.unlinkOnlyDesc', { other: displayName(MOTHER) })
      );
      await profile.dialog
        .getByRole('button', {
          name: profile.t('profile.family.unlinkOnlyBtn', { name: displayName(FATHER) }),
        })
        .click();

      await expect(
        profile.toast(profile.t('profile.family.parentUnlinkedOnlyToast'))
      ).toBeVisible();
      await expect(profile.person(displayName(MOTHER))).toBeVisible();
      await expect(profile.person(displayName(FATHER))).toHaveCount(0);
      const [move] = api.find('POST', '/move-to-family');
      expect(move.body).toMatchObject({
        partner1_id: MOTHER.id,
        from_family_id: 'fam-parents',
      });

      await profile.undo();
      await expect(profile.person(displayName(FATHER))).toBeVisible();
      await expect(profile.person(displayName(MOTHER))).toBeVisible();
    });

    test('two-parent family: "unlink from both parents" detaches the whole family, undo restores', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await profile.unlinkParentButtons.last().click();
      await expect(profile.dialog).toContainText(
        profile.t('profile.family.unlinkParentChoiceTitle', { name: displayName(MOTHER) })
      );
      await profile.dialog
        .getByRole('button', { name: profile.t('profile.family.unlinkBothBtn') })
        .click();

      await expect(profile.toast(profile.t('profile.family.parentUnlinkedToast'))).toBeVisible();
      await expect(profile.person(displayName(FATHER))).toHaveCount(0);
      await expect(profile.person(displayName(MOTHER))).toHaveCount(0);
      expect(api.find('DELETE', `/children/${SUBJECT.id}`)).toHaveLength(1);

      await profile.undo();
      await expect(profile.person(displayName(FATHER))).toBeVisible();
      await expect(profile.person(displayName(MOTHER))).toBeVisible();
    });

    test('single-parent family: parent unlink shows a simple confirm dialog', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page, { singleParent: true });
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await profile.unlinkParentButtons.first().click();
      await expect(profile.dialog).toContainText(profile.t('profile.family.unlinkParentTitle'));
      // No two-option choice is offered.
      await expect(
        profile.dialog.getByRole('button', { name: profile.t('profile.family.unlinkBothBtn') })
      ).toHaveCount(0);
      await profile.confirmRemoval();

      await expect(profile.toast(profile.t('profile.family.parentUnlinkedToast'))).toBeVisible();
      await expect(profile.person(displayName(FATHER))).toHaveCount(0);
      expect(api.find('DELETE', `/children/${SUBJECT.id}`)).toHaveLength(1);

      await profile.undo();
      await expect(profile.person(displayName(FATHER))).toBeVisible();
    });

    test('Change parents: pick a new partner pair via search, submit, undo', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'editor' });
      const api = await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await profile.changeParentsButton.click();
      await expect(profile.dialog).toContainText(profile.t('profile.family.changeParentsTitle'));
      await profile.pickParent('parent1Label', 'Boris', displayName(BORIS));
      await profile.pickParent('parent2Label', 'Galina', displayName(GALINA));
      await profile.submitChangeParents();

      await expect(profile.toast(profile.t('profile.family.moveSuccessToast'))).toBeVisible();
      await expect(profile.dialog).toHaveCount(0);
      await expect(profile.person(displayName(BORIS))).toBeVisible();
      await expect(profile.person(displayName(GALINA))).toBeVisible();
      await expect(profile.person(displayName(FATHER))).toHaveCount(0);
      const [move] = api.find('POST', '/move-to-family');
      expect(move.body).toMatchObject({ partner1_id: BORIS.id, partner2_id: GALINA.id });

      await profile.undo();
      await expect(profile.person(displayName(FATHER))).toBeVisible();
      await expect(profile.person(displayName(MOTHER))).toBeVisible();
      await expect(profile.person(displayName(BORIS))).toHaveCount(0);
    });

    test('Change parents: cycle error is shown inline and the dialog stays open', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'editor' });
      await installFamilyApiMock(page);
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      await profile.changeParentsButton.click();
      await profile.pickParent('parent1Label', 'Dmitri', displayName(DMITRI));
      await profile.submitChangeParents();

      await expect(profile.dialog.getByRole('alert')).toContainText(
        profile.t('errors.cycle_detected')
      );
      await expect(profile.dialog).toBeVisible();
      // Original parents are untouched.
      await expect(profile.person(displayName(FATHER))).toBeVisible();
    });

    for (const [key, relation] of [
      ['addSibling', 'sibling'],
      ['addSpouse', 'spouse'],
      ['addChild', 'child'],
    ] as const) {
      test(`quick-add "${relation}": Link Existing lists server candidates, filters and links`, async ({
        page,
        mockAuthenticatedState,
      }) => {
        await mockAuthenticatedState({ role: 'editor' });
        const api = await installFamilyApiMock(page);
        const profile = new PersonProfilePage(page, locale);
        await profile.open(SUBJECT.id);

        await profile.quickAddButton(key).click();
        await profile.switchQuickAddToLinkExisting();
        const combo = profile.existingPersonCombobox();
        await combo.click();

        // Regression: candidates come from server search, not an empty preload.
        await expect(profile.options().first()).toBeVisible();
        await expect(profile.noResults()).toHaveCount(0);
        await expect(profile.options().filter({ hasText: displayName(GALINA) })).toBeVisible();
        await expect(profile.options().filter({ hasText: displayName(BORIS) })).toBeVisible();
        // The anchor person is never offered as their own relative.
        await expect(profile.options().filter({ hasText: displayName(SUBJECT) })).toHaveCount(0);

        // Typing filters the list through the directory search.
        await combo.fill('Galina');
        await expect(profile.options()).toHaveCount(1);
        await expect(profile.options().first()).toContainText(displayName(GALINA));
        expect(api.requests.some((r) => r.path.includes('q=Galina'))).toBe(true);

        // A query without matches shows the empty message.
        await combo.fill('Zzzz');
        await expect(profile.noResults()).toBeVisible();

        await combo.fill('Galina');
        await profile.options().first().click();
        await profile.dialog
          .getByRole('button', { name: profile.t('person.quickAdd.submitLinkBtn') })
          .click();

        await expect(profile.toast(profile.t('person.quickAdd.addedToast'))).toBeVisible();
        await expect(profile.dialog).toHaveCount(0);
        expect(api.find('POST', '/quick-add')[0].body).toMatchObject({
          relation,
          existing_person_id: GALINA.id,
        });
        await expect(profile.person(displayName(GALINA))).toBeVisible();
      });
    }

    test('viewers do not see unlink or Change parents controls', async ({
      page,
      mockAuthenticatedState,
    }) => {
      await mockAuthenticatedState({ role: 'viewer' });
      await installFamilyApiMock(page, { canEdit: false });
      const profile = new PersonProfilePage(page, locale);
      await profile.open(SUBJECT.id);

      // Relatives are still readable...
      await expect(profile.person(displayName(FATHER))).toBeVisible();
      await expect(profile.person(displayName(SPOUSE))).toBeVisible();
      // ...but no editing controls are rendered.
      await expect(profile.unlinkParentButtons).toHaveCount(0);
      await expect(profile.unlinkSpouseButton).toHaveCount(0);
      await expect(profile.unlinkChildButton).toHaveCount(0);
      await expect(profile.changeParentsButton).toHaveCount(0);
      await expect(profile.quickAddButton('addSibling')).toHaveCount(0);
    });
  });
}
