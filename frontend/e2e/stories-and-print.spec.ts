import { test, expect } from './fixtures/test-base';
import {
  PrintStudioPage,
  SecurityMcpPage,
  StoryIntakePage,
} from './pages/stories-and-print.page';

test.describe('Milestone 3.1 & 3.2 — Story Intake, MCP Tokens & Print Studio', () => {
  test.beforeEach(async ({ mockAuthenticatedState, page }) => {
    await mockAuthenticatedState({
      id: 'usr-admin-01',
      email: 'admin@genealogy.local',
      display_name: 'Admin User',
      role: 'admin',
      start_page: 'home',
    });

    await page.route('**/api/v1/people*', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          items: [
            {
              id: 'node-root',
              gid: 'I0001',
              display_name: 'Anna Sergeyevna Volkova',
              given_name: 'Anna',
              patronymic: 'Sergeyevna',
              surname: 'Volkova',
              sex: 'F',
              is_living: true,
            },
            {
              id: 'node-mother-1',
              gid: 'I0002',
              display_name: 'Maria Ivanovna Volkova',
              given_name: 'Maria',
              patronymic: 'Ivanovna',
              surname: 'Volkova',
              sex: 'F',
              is_living: true,
            },
          ],
          total: 2,
          next_cursor: null,
        }),
      });
    });
  });

  test('parses family story into staged draft subtree, reconciles link_enrich, and commits to tree', async ({
    page,
  }) => {
    let parsePayload: Record<string, unknown> | null = null;
    let commitPayload: {
      match_overrides?: Array<{
        draft_person_id: string;
        selected_action: string;
        target_person_id?: string;
      }>;
    } | null = null;

    await page.route('**/api/v1/ai/parse-story', async (route) => {
      parsePayload = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'draft-e2e-1',
          title: 'Volkova Family Narrative',
          speaker_id: 'node-root',
          speaker_person_id: 'node-root',
          is_new_speaker: false,
          raw_text:
            'My name is Anna Sergeyevna Volkova, born in 1975 in Tver. My father Pyotr Fyodorovich Shcheglov was born in 1945 in Tver.',
          status: 'draft',
          extracted_data: {
            draft_people: [
              {
                temp_id: 'temp-1',
                given_name: 'Anna',
                patronymic: 'Sergeyevna',
                surname: 'Volkova',
                sex: 'F',
                birth_date: '1975',
                birth_place: 'Tver',
                is_living: true,
                role_to_speaker: 'speaker',
                confidence_score: 96,
              },
              {
                temp_id: 'temp-2',
                given_name: 'Pyotr',
                patronymic: 'Fyodorovich',
                surname: 'Shcheglov',
                sex: 'M',
                birth_date: '1945',
                birth_place: 'Tver',
                is_living: true,
                role_to_speaker: 'father',
                confidence_score: 91,
              },
            ],
            draft_unions: [],
            draft_parent_child: [
              {
                parent_temp_id: 'temp-2',
                child_temp_id: 'temp-1',
                relation_type: 'biological',
              },
            ],
            draft_events: [],
            reconciliation: [
              {
                draft_temp_id: 'temp-1',
                recommended_action: 'link_enrich',
                candidate_person_id: 'node-root',
                candidate_person_gid: 'I0001',
                candidate_display_name: 'Anna Sergeyevna Volkova',
                confidence: 98,
                reasons: ['speaker_anchor', 'vital_dates'],
                enrich_fields: ['birth_place', 'notes'],
              },
              {
                draft_temp_id: 'temp-2',
                recommended_action: 'create_new',
                confidence: 0,
                reasons: [],
                enrich_fields: [],
              },
            ],
          },
          created_at: '2026-11-07T10:00:00Z',
          updated_at: '2026-11-07T10:00:00Z',
        }),
      });
    });

    await page.route('**/api/v1/ai/drafts/draft-e2e-1/commit', async (route) => {
      commitPayload = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          draft_id: 'draft-e2e-1',
          created_count: 1,
          linked_count: 1,
          focal_person_id: null,
          tx_id: 'tx-story-e2e-1',
          transaction_id: 'tx-story-e2e-1',
          created_person_ids: [],
          updated_person_ids: [],
          status: 'committed',
        }),
      });
    });

    const storyPage = new StoryIntakePage(page);
    await storyPage.goto();

    // 1. Verify 3-step progression bar
    await expect(storyPage.pageContainer).toBeVisible();
    await expect(storyPage.stepTracker).toBeVisible();
    await expect(
      storyPage.stepTracker.getByText(/1\. Narrate Story|1\. Семейный рассказ/i)
    ).toBeVisible();
    await expect(
      storyPage.stepTracker.getByText(/2\. Review Draft Tree|2\. Проверка ветви/i)
    ).toBeVisible();
    await expect(
      storyPage.stepTracker.getByText(/3\. Merge into Tree|3\. Внесение в древо/i)
    ).toBeVisible();

    // 2. Verify speaker selector card
    await expect(storyPage.speakerSelectorCard).toBeVisible();

    // 3. Verify extraction engine toggle (Gemini 3.8 Flash vs Built-in Offline)
    await expect(storyPage.engineSelectorBar).toBeVisible();
    await expect(storyPage.geminiEngineOption).toBeVisible();
    await expect(storyPage.builtinEngineOption).toBeVisible();

    await storyPage.builtinEngineOption.click();
    await expect(storyPage.builtinEngineOption).toHaveAttribute(
      'aria-checked',
      'true'
    );
    await storyPage.geminiEngineOption.click();
    await expect(storyPage.geminiEngineOption).toHaveAttribute(
      'aria-checked',
      'true'
    );

    // 4. Fill story narrative and parse draft subtree
    const storyText =
      'My name is Anna Sergeyevna Volkova, born in 1975 in Tver. My father Pyotr Fyodorovich Shcheglov was born in 1945 in Tver.';
    await storyPage.narrativeTextarea.fill(storyText);
    await expect(storyPage.extractButton).toBeEnabled();
    await storyPage.extractButton.click();

    // 5. Verify staged draft canvas and reconciliation cards (link_enrich mapped to Link & Enrich)
    await expect(storyPage.stagedWorkspace).toBeVisible();
    await expect(storyPage.draftCanvas).toBeVisible();
    await expect(storyPage.reconciliationCard).toBeVisible();

    expect(parsePayload).toMatchObject({
      raw_text: storyText,
      engine: 'gemini',
    });

    const speakerReconcileRow = storyPage.getReconcileRow('temp-1');
    const fatherReconcileRow = storyPage.getReconcileRow('temp-2');
    await expect(speakerReconcileRow).toBeVisible();
    await expect(fatherReconcileRow).toBeVisible();

    const linkActionRadio = speakerReconcileRow.getByRole('radio', {
      name: /Enrich Existing|Дополнить профиль/i,
    });
    await expect(linkActionRadio).toHaveAttribute('aria-checked', 'true');
    await expect(speakerReconcileRow).toContainText('I0001');

    // 6. Commit draft via /api/v1/ai/drafts/{id}/commit
    await storyPage.commitButton.click();
    await expect(storyPage.stagedWorkspace).toBeHidden();

    expect(commitPayload).not.toBeNull();
    const overrides = commitPayload!.match_overrides;
    expect(overrides).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          draft_person_id: 'temp-1',
          selected_action: 'link_enrich',
          target_person_id: 'node-root',
        }),
        expect.objectContaining({
          draft_person_id: 'temp-2',
          selected_action: 'create_new',
        }),
      ])
    );
  });

  test('creates MCP Personal Access Token, shows one-time secret banner, and opens revocation modal', async ({
    page,
  }) => {
    await page.route('**/api/v1/auth/passkeys', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ items: [] }),
      });
    });

    await page.route('**/api/v1/auth/sessions', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ items: [] }),
      });
    });

    await page.route('**/api/v1/mcp/tokens', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ items: [] }),
        });
        return;
      }

      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            token: 'mcp_pat_test123',
            token_info: {
              id: 'tok-1',
              name: 'Claude Desktop',
              token_prefix: 'mcp_pat_test',
              scopes: ['read:tree', 'write:draft'],
              created_at: '2026-11-07T10:00:00Z',
            },
          }),
        });
      }
    });

    await page.route('**/api/v1/mcp/tokens/tok-1', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ status: 'revoked' }),
      });
    });

    const securityPage = new SecurityMcpPage(page);
    await securityPage.goto();

    await expect(securityPage.mcpPanel).toBeVisible();

    // Create a new MCP token
    await securityPage.tokenNameInput.fill('Claude Desktop');
    await expect(securityPage.createTokenButton).toBeEnabled();
    await securityPage.createTokenButton.click();

    // Verify one-time secret banner and token row render without crashing on scopes.map
    await expect(securityPage.freshTokenBanner).toBeVisible();
    await expect(securityPage.rawTokenValue).toHaveText('mcp_pat_test123');

    const tokenRow = securityPage.getTokenRow('tok-1');
    await expect(tokenRow).toBeVisible();
    await expect(tokenRow).toContainText('Claude Desktop');
    await expect(tokenRow).toContainText('read:tree');
    await expect(tokenRow).toContainText('write:draft');

    // Click Revoke and verify confirmation modal opens
    const revokeBtn = securityPage.getRevokeButton('tok-1');
    await revokeBtn.click();

    await expect(securityPage.dialog).toBeVisible();
    await expect(securityPage.dialog).toContainText('Claude Desktop');
    await expect(securityPage.confirmRevokeButton).toBeVisible();

    // Confirm revocation
    await securityPage.confirmRevokeButton.click();
    await expect(securityPage.dialog).toBeHidden();
    await expect(tokenRow).toBeHidden();
  });

  test('renders Print Studio configuration sidebar and interactive SVG preview canvas', async ({
    page,
  }) => {
    await page.route('**/api/v1/export/pdf/preview-info', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          total_sheets: 4,
          grid_rows: 2,
          grid_cols: 2,
          canvas_width_mm: 594,
          canvas_height_mm: 420,
          sheet_width_mm: 297,
          sheet_height_mm: 210,
          overlap_mm: 10,
          format: 'tiled',
          target_sheet_size: 'A4',
          total_persons: 7,
          summary_text: '4 Sheets (2×2 Grid)',
        }),
      });
    });

    const printPage = new PrintStudioPage(page);
    await printPage.goto();

    await expect(printPage.pageContainer).toBeVisible();
    await expect(printPage.configSidebar).toBeVisible();
    await expect(printPage.downloadButton).toBeVisible();

    // On mobile viewports (<1024px), switch to the Preview tab to inspect the SVG canvas
    await printPage.ensurePreviewCanvasVisible();

    await expect(printPage.previewCanvasRoot).toBeVisible();
    await expect(printPage.previewCanvasRoot.locator('svg').first()).toBeVisible();
  });
});
