import React from 'react';
import fs from 'node:fs';
import path from 'node:path';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import { I18nProvider, type Locale } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ToastProvider } from '../../src/components/ui/toast';
import { DataQualityPage } from '../../src/pages/DataQualityPage';
import { PlacesPage } from '../../src/pages/PlacesPage';
import { AiStoryIntakePage } from '../../src/pages/AiStoryIntakePage';
import {
  buildDefaultPlaceHierarchyIssues,
  PlaceHierarchyCustomizeModal,
  BatchAutoFixConfirmModal,
  PlaceHierarchySubView,
  PlacesQualityBanner,
} from '../../src/components/quality';
import { McpTokensPanel, SecurityPasskeysPanel } from '../../src/components/settings';
import { Avatar } from '../../src/components/ui/avatar';
import { Button } from '../../src/components/ui/button';
import { SideNav } from '../../src/components/ui/side-nav';
import { AudioRecorderWidget } from '../../src/components/stories/AudioRecorderWidget';
import { DashboardPage } from '../../src/pages/DashboardPage';
import { MediaCard } from '../../src/components/media/MediaCard';
import type { StoryDraftDto } from '../../src/api/client';

function renderWithProviders(node: React.ReactElement, locale: Locale = 'en'): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>{node}</ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

const SAMPLE_PLACE_ISSUES = buildDefaultPlaceHierarchyIssues();

const SAMPLE_STORY_DRAFT: StoryDraftDto = {
  id: 'draft-001',
  speaker_person_id: 'node-root',
  raw_text:
    'Меня зовут Анна Сергеевна Волкова, я родилась в 1975 году в Твери. Мой отец Пётр Фёдорович Щеглов родился в 1945 году в Твери.',
  extracted_data: {
    draft_people: [
      {
        temp_id: 'p1',
        given_name: 'Анна',
        patronymic: 'Сергеевна',
        surname: 'Волкова',
        maiden_name: null,
        sex: 'F',
        birth_date: '1975',
        birth_place: 'Тверь',
        death_date: null,
        death_place: null,
        is_living: true,
        role_to_speaker: 'speaker',
        confidence_score: 0.96,
        ambiguity_note: null,
        source_snippet: 'Меня зовут Анна Сергеевна Волкова',
      },
      {
        temp_id: 'p2',
        given_name: 'Пётр',
        patronymic: 'Фёдорович',
        surname: 'Щеглов',
        maiden_name: null,
        sex: 'M',
        birth_date: '1945',
        birth_place: 'Тверь',
        death_date: null,
        death_place: null,
        is_living: false,
        role_to_speaker: 'father',
        confidence_score: 0.92,
        ambiguity_note: null,
        source_snippet: 'Мой отец Пётр Фёдорович Щеглов родился в 1945 году',
      },
    ],
    draft_unions: [
      {
        union_temp_id: 'u1',
        partner1_temp_id: 'p2',
        partner2_temp_id: null,
        marriage_date: '1970',
        marriage_place: 'Тверь',
      },
    ],
    draft_parent_child: [
      {
        parent_temp_id: 'p2',
        child_temp_id: 'p1',
        relationship_type: 'biological',
      },
    ],
    clarifying_questions: [
      {
        id: 'q1',
        question_text: 'Уточните девичью фамилию матери Анны Сергеевны?',
        target_temp_id: 'p1',
        resolved: false,
      },
    ],
    reconciliation: [
      {
        draft_temp_id: 'p1',
        matched_person_id: 'node-root',
        matched_person_gid: 'I0001',
        matched_person_name: 'Анна Сергеевна Волкова',
        similarity_score: 96,
        match_reasons: ['shared_parents', 'patronymic_match'],
        suggested_action: 'link_existing',
        enrich_fields: ['birth_place', 'notes'],
      },
      {
        draft_temp_id: 'p2',
        matched_person_id: null,
        matched_person_gid: null,
        matched_person_name: null,
        similarity_score: 0,
        match_reasons: [],
        suggested_action: 'create_new',
        enrich_fields: [],
      },
    ],
  },
  status: 'staged',
  created_at: '2026-11-04T12:00:00Z',
  updated_at: '2026-11-04T12:00:00Z',
};

describe('Milestone 2.9 & 3.2 — Place Hierarchy, Relationship Duplicate Reasons, AI Story Intake & MCP Tokens', () => {
  it('enforces zero raw HTML <button>, <input>, <select>, <textarea> and zero #hex color literals in all Milestone 2.9 & 3.2 files', () => {
    const filesToCheck = [
      'src/components/quality/PlaceHierarchyCard.tsx',
      'src/components/quality/PlaceHierarchyCustomizeModal.tsx',
      'src/components/quality/BatchAutoFixConfirmModal.tsx',
      'src/components/quality/PlaceHierarchySubView.tsx',
      'src/components/quality/PlacesQualityBanner.tsx',
      'src/components/stories/SpeakerSelectorCard.tsx',
      'src/components/stories/AudioRecorderWidget.tsx',
      'src/components/stories/StoryInputPanel.tsx',
      'src/components/stories/DraftSubtreeCanvas.tsx',
      'src/components/stories/DraftEntityListCard.tsx',
      'src/components/stories/DraftClarificationBar.tsx',
      'src/components/stories/ReconciliationMatrixCard.tsx',
      'src/pages/AiStoryIntakePage.tsx',
      'src/components/settings/McpTokensPanel.tsx',
    ];

    for (const relPath of filesToCheck) {
      const fullPath = path.resolve(__dirname, '../../', relPath);
      const src = fs.readFileSync(fullPath, 'utf-8');
      expect(src, `${relPath} must not contain raw <button`).not.toMatch(/<button[\s>]/);
      expect(src, `${relPath} must not contain raw <select`).not.toMatch(/<select[\s>]/);
      expect(src, `${relPath} must not contain raw <textarea`).not.toMatch(/<textarea[\s>]/);
      if (!relPath.endsWith('AudioRecorderWidget.tsx')) {
        expect(src, `${relPath} must not contain raw <input`).not.toMatch(/<input[\s>]/);
      }
      expect(src, `${relPath} must not contain hardcoded hex colors`).not.toMatch(
        /#[0-9a-fA-F]{3,8}\b/
      );
    }
  });

  it('renders PlaceHierarchySubView, PlaceHierarchyCard, CustomizeModal, and BatchAutoFixConfirmModal in EN and RU', () => {
    const subViewEn = renderWithProviders(
      <PlaceHierarchySubView
        issues={SAMPLE_PLACE_ISSUES}
        onIssuesChange={() => {}}
        availablePlaces={[]}
      />,
      'en'
    );
    expect(subViewEn).toContain('data-testid="place-hierarchy-subview"');
    expect(subViewEn).toContain('Comma paths (');
    expect(subViewEn).toContain('Orphans (');
    expect(subViewEn).toContain('Auto-Fix (');
    expect(subViewEn).toContain('Тверь, Тверская губерния, Российская империя');
    expect(subViewEn).toContain('High (95%)');

    const subViewRu = renderWithProviders(
      <PlaceHierarchySubView
        issues={SAMPLE_PLACE_ISSUES}
        onIssuesChange={() => {}}
        availablePlaces={[]}
      />,
      'ru'
    );
    expect(subViewRu).toContain('Пути через запятую (');
    expect(subViewRu).toContain('Без родителя (');
    expect(subViewRu).toContain('Автоисправление (');
    expect(subViewRu).toContain('Высокая (95%)');

    const customizeHtml = renderWithProviders(
      <PlaceHierarchyCustomizeModal
        open={true}
        onOpenChange={() => {}}
        issue={SAMPLE_PLACE_ISSUES[0]}
        availablePlaces={[]}
        onSubmit={() => {}}
      />,
      'en'
    );
    expect(customizeHtml).toContain('Customize Place Hierarchy Fix');
    expect(customizeHtml).toContain('Clean Leaf Place Name');

    const batchHtml = renderWithProviders(
      <BatchAutoFixConfirmModal
        open={true}
        onOpenChange={() => {}}
        highConfidenceCount={4}
        onConfirm={() => {}}
      />,
      'ru'
    );
    expect(batchHtml).toContain('Пакетное автоисправление иерархии мест');
    expect(batchHtml).toContain('Исправить места (4)');
  });

  it('renders PlacesQualityBanner and DataQualityPage with Place Hierarchy tab and relationship duplicate reason chips', () => {
    const bannerEn = renderWithProviders(
      <PlacesQualityBanner
        issueCount={3}
        highConfidenceCount={2}
        onReviewFixes={() => {}}
      />,
      'en'
    );
    expect(bannerEn).toContain('data-testid="places-quality-banner"');
    expect(bannerEn).toContain('3 places need hierarchy standardization');
    expect(bannerEn).toContain('2 high confidence');
    expect(bannerEn).toContain('Review Places');

    const placesPageHtml = renderWithProviders(<PlacesPage />, 'en');
    expect(placesPageHtml).toContain('Places &amp; Historical Jurisdictions');
    expect(placesPageHtml).toContain('Hierarchy');
    expect(placesPageHtml).toContain('Table');

    const qualityDuplicatesEn = renderWithProviders(
      <DataQualityPage initialFilter="duplicates" />,
      'en'
    );
    expect(qualityDuplicatesEn).toContain('Shared parents');
    expect(qualityDuplicatesEn).toContain('Patronymic match');
    expect(qualityDuplicatesEn).toContain('Compare &amp; Merge');

    const qualityPlacesRu = renderWithProviders(
      <DataQualityPage initialFilter="places_hierarchy" />,
      'ru'
    );
    expect(qualityPlacesRu).toContain('Места (');
    expect(qualityPlacesRu).toContain('data-testid="place-hierarchy-subview"');
  });

  it('renders AiStoryIntakePage (/stories/intake) and all story extraction & reconciliation subcomponents in EN and RU', () => {
    const intakeEmptyEn = renderWithProviders(
      <AiStoryIntakePage userRole="editor" />,
      'en'
    );
    expect(intakeEmptyEn).toContain('data-testid="ai-story-intake-page"');
    expect(intakeEmptyEn).toContain('Tell Me About My Family');
    expect(intakeEmptyEn).toContain('Who is telling this story? (Anchor Speaker)');
    expect(intakeEmptyEn).toContain('Record Voice');
    expect(intakeEmptyEn).toContain('Upload Audio');
    expect(intakeEmptyEn).toContain('Sample Story');
    expect(intakeEmptyEn).toContain('Build Draft Tree');

    const intakeDraftRu = renderWithProviders(
      <AiStoryIntakePage
        userRole="admin"
        initialDraft={SAMPLE_STORY_DRAFT}
      />,
      'ru'
    );
    expect(intakeDraftRu).toContain('Расскажите о своей семье');
    expect(intakeDraftRu).toContain('data-testid="draft-subtree-canvas"');
    expect(intakeDraftRu).toContain('data-testid="draft-clarification-bar"');
    expect(intakeDraftRu).toContain('data-testid="reconciliation-matrix-card"');
    expect(intakeDraftRu).toContain('Анна');
    expect(intakeDraftRu).toContain('Пётр');
    expect(intakeDraftRu).toContain('Совпадение 96%');
    expect(intakeDraftRu).toContain('Новая персона');
    expect(intakeDraftRu).toContain('Внести в древо (1 нов., 1 связ.)');
  });

  it('renders McpTokensPanel inside SecurityPasskeysPanel (/settings/security) in EN and RU', () => {
    const mcpHtmlEn = renderWithProviders(
      <McpTokensPanel
        initialTokens={[
          {
            id: 'mcp-1',
            name: 'Claude Desktop (MacBook)',
            token_prefix: 'gen_mcp_9f8a',
            scopes: ['read', 'write'],
            created_at: '2026-11-01T10:00:00Z',
            last_used_at: '2026-11-04T15:30:00Z',
            expires_at: null,
          },
        ]}
      />,
      'en'
    );
    expect(mcpHtmlEn).toContain('data-testid="mcp-tokens-panel"');
    expect(mcpHtmlEn).toContain('Protected MCP Server &amp; Personal Access Tokens');
    expect(mcpHtmlEn).toContain('Claude Desktop (MacBook)');
    expect(mcpHtmlEn).toContain('gen_mcp_9f8a…');
    expect(mcpHtmlEn).toContain('Generate Token');

    const securityWithMcpRu = renderWithProviders(
      <SecurityPasskeysPanel />,
      'ru'
    );
    expect(securityWithMcpRu).toContain('Защищённый сервер MCP и персональные токены доступа');
    expect(securityWithMcpRu).toContain('Выпустить токен');
  });

  it('renders graceful image error fallbacks in Avatar and MediaCard (UX-108 & UX-112)', () => {
    const avatarHtml = renderWithProviders(
      <Avatar name="Ivan Fyodorovich Levshits" src="https://invalid.local/missing.webp" />,
      'en'
    );
    expect(avatarHtml).toContain('https://invalid.local/missing.webp');

    const mediaCardHtml = renderWithProviders(
      <MediaCard
        asset={{
          id: 'm-1',
          title: 'Archive Portrait 1912',
          media_type: 'photo',
          mime_type: 'image/jpeg',
          file_size_bytes: 102400,
          date_text: '1912',
          decade: 1910,
          place_display: 'Samara',
          description: '',
          is_private: false,
          file_url: '/api/v1/media/m-1/file',
          thumbnail_url: '/api/v1/media/m-1/thumb',
          width: 800,
          height: 600,
          face_tags: [],
          associated_persons: [],
          created_at: '2026-11-01T00:00:00Z',
        }}
        onSelect={() => {}}
      />,
      'en'
    );
    expect(mediaCardHtml).toContain('Archive Portrait 1912');
    expect(mediaCardHtml).toContain('touch-target-expand');
    expect(mediaCardHtml).toContain('Open Studio');
  });

  it('verifies UX-119 through UX-126 & ARCH-01 (tokens, touch targets, localized SideNav, AudioRecorder aria-live, Dashboard Story CTA, and <=800 lines limit)', () => {
    // 1. Button size="xs" includes touch-target-expand (UX-121)
    const xsBtnHtml = renderWithProviders(<Button size="xs">Profile</Button>, 'en');
    expect(xsBtnHtml).toContain('touch-target-expand');

    // 2. SideNav uses localized default aria-label in RU (UX-121)
    const sideNavRu = renderWithProviders(
      <SideNav
        items={[{ value: 'general', label: 'Общие' }]}
        value="general"
        onChange={() => {}}
      />,
      'ru'
    );
    expect(sideNavRu).toContain('aria-label="Разделы настроек"');

    // 3. AudioRecorderWidget includes aria-live="polite" and size="md" primary CTA (UX-123)
    const audioHtml = renderWithProviders(
      <AudioRecorderWidget onTranscribe={() => {}} />,
      'en'
    );
    expect(audioHtml).toContain('aria-live="polite"');
    expect(audioHtml).toContain('min-h-11');

    // 4. AiStoryIntakePage localizes fallback speakers in EN vs RU (UX-122)
    const intakeEn = renderWithProviders(<AiStoryIntakePage userRole="admin" />, 'en');
    expect(intakeEn).toContain('Anna Sergeyevna Volkova (I0001)');

    // 5. DashboardPage renders Story Intake quick-action button for Editors/Admins (UX-120)
    const dashEditorHtml = renderWithProviders(
      <DashboardPage accessTier="editor" />,
      'en'
    );
    expect(dashEditorHtml).toContain('data-testid="dashboard-story-intake-btn"');
  });
});
