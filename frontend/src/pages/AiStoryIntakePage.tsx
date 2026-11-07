import React, { useCallback, useEffect, useState } from 'react';
import { CheckCircle2, Sparkles } from 'lucide-react';
import {
  AccessDeniedCard,
  Badge,
  Card,
  PageHeader,
  Spinner,
  useToast,
} from '../components/ui';
import {
  DraftClarificationBar,
  DraftEntityListCard,
  DraftSubtreeCanvas,
  ReconciliationMatrixCard,
  ReconciliationResolutionState,
  SpeakerOption,
  SpeakerSelectorCard,
  StoryInputPanel,
} from '../components/stories';
import {
  DraftPersonNodeDto,
  fetchPeopleDirectory,
  historyApi,
  storyApi,
  StoryDraftDto,
} from '../api/client';
import {
  getExtractionEngine,
  setExtractionEngine,
  StoryExtractionEngine,
} from '../api/gemini-story-extractor';
import { useAuth } from '../lib/auth-context';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';

export interface AiStoryIntakePageProps {
  initialDraft?: StoryDraftDto | null;
  initialSpeakers?: SpeakerOption[];
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

const DEFAULT_SPEAKERS: SpeakerOption[] = [
  { id: 'node-root', gid: 'I0001', display_name: 'Анна Сергеевна Волкова' },
  { id: 'node-mother-1', gid: 'I0002', display_name: 'Мария Ивановна Волкова' },
  { id: 'node-deceased-1', gid: 'I0012', display_name: 'Пётр Фёдорович Щеглов' },
];

export const AiStoryIntakePage: React.FC<AiStoryIntakePageProps> = ({
  initialDraft = null,
  initialSpeakers,
  userRole,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const canEdit = effectiveRole === 'admin' || effectiveRole === 'editor';

  const [speakers, setSpeakers] = useState<SpeakerOption[]>(
    initialSpeakers ?? DEFAULT_SPEAKERS
  );
  const [selectedSpeakerId, setSelectedSpeakerId] = useState<string>(
    user?.person_id || user?.home_person_id || DEFAULT_SPEAKERS[0].id
  );
  const [isNewSpeaker, setIsNewSpeaker] = useState<boolean>(false);

  const [narrativeText, setNarrativeText] = useState<string>(
    initialDraft?.raw_text ?? ''
  );
  const [language, setLanguage] = useState<string>('auto');
  const [transcribing, setTranscribing] = useState<boolean>(false);
  const [extracting, setExtracting] = useState<boolean>(false);
  const [refining, setRefining] = useState<boolean>(false);
  const [committing, setCommitting] = useState<boolean>(false);

  const [engine, setEngine] = useState<StoryExtractionEngine>(() =>
    getExtractionEngine()
  );

  useEffect(() => {
    if (typeof window !== 'undefined' && window.localStorage) {
      try {
        localStorage.removeItem('genealogy_gemini_api_key');
      } catch {
        // Ignore
      }
    }
  }, []);

  const handleEngineChange = (nextEngine: StoryExtractionEngine) => {
    setEngine(nextEngine);
    setExtractionEngine(nextEngine);
  };

  const [activeDraft, setActiveDraft] = useState<StoryDraftDto | null>(
    initialDraft
  );
  const [editingPerson, setEditingPerson] = useState<DraftPersonNodeDto | null>(
    null
  );
  const [resolutions, setResolutions] = useState<
    Record<string, ReconciliationResolutionState>
  >(() => {
    if (!initialDraft) return {};
    const map: Record<string, ReconciliationResolutionState> = {};
    for (const person of initialDraft.extracted_data.draft_people) {
      const cand = initialDraft.extracted_data.reconciliation?.find(
        (c) => c.draft_temp_id === person.temp_id
      );
      map[person.temp_id] = {
        action:
          cand?.matched_person_id && cand.similarity_score >= 60
            ? 'link_existing'
            : 'create_new',
        matched_person_id: cand?.matched_person_id ?? null,
        enrich_fields: cand?.enrich_fields ?? ['birth_place', 'notes'],
      };
    }
    return map;
  });

  const navigateTo = useCallback(
    (nextPath: string) => {
      if (onNavigate) {
        onNavigate(nextPath);
      } else if (typeof window !== 'undefined') {
        window.history.pushState({}, '', nextPath);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }
    },
    [onNavigate]
  );

  useEffect(() => {
    if (!canEdit || initialSpeakers) return;
    let cancelled = false;
    fetchPeopleDirectory({ limit: 30 })
      .then((res) => {
        if (!cancelled && Array.isArray(res.items) && res.items.length > 0) {
          setSpeakers(
            res.items.map((item) => ({
              id: item.id,
              gid: item.gid,
              display_name: item.display_name,
            }))
          );
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [canEdit, initialSpeakers]);

  const syncResolutionsFromDraft = (draft: StoryDraftDto) => {
    const next: Record<string, ReconciliationResolutionState> = {};
    for (const person of draft.extracted_data.draft_people) {
      const cand = draft.extracted_data.reconciliation?.find(
        (c) => c.draft_temp_id === person.temp_id
      );
      next[person.temp_id] = {
        action:
          cand?.matched_person_id && cand.similarity_score >= 60
            ? 'link_existing'
            : 'create_new',
        matched_person_id: cand?.matched_person_id ?? null,
        enrich_fields: cand?.enrich_fields ?? ['birth_place', 'notes'],
      };
    }
    setResolutions(next);
  };

  if (!canEdit) {
    return (
      <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-12">
        <AccessDeniedCard
          currentRole="viewer"
          onOpenProfile={() => navigateTo('/settings/profile')}
          onBackToTree={() => navigateTo('/tree')}
        />
      </div>
    );
  }

  const handleTranscribeAudio = async (
    blob: Blob | File,
    speechTranscript?: string
  ) => {
    setTranscribing(true);
    try {
      const res = await storyApi.transcribeAudio(
        blob,
        speechTranscript,
        language === 'auto' ? undefined : language
      );
      const transcribed = (res.text || res.transcript || speechTranscript || '').trim();
      if (transcribed) {
        setNarrativeText((prev) => {
          const current = (prev ?? '').trim();
          return current ? `${current}\n\n${transcribed}` : transcribed;
        });
        addToast({
          type: 'success',
          title: t('stories.toast.transcribedTitle'),
        });
      } else {
        addToast({
          type: 'info',
          title: t('common.errorLoading'),
          description: t('stories.audio.noTranscriptError'),
        });
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setTranscribing(false);
    }
  };

  const handleExtractSubtree = async () => {
    if (!narrativeText.trim() || extracting) return;
    setExtracting(true);

    try {
      const draft = await storyApi.parseStory({
        speaker_id: isNewSpeaker ? null : selectedSpeakerId,
        speaker_person_id: isNewSpeaker ? null : selectedSpeakerId,
        is_new_speaker: isNewSpeaker,
        raw_text: narrativeText.trim(),
        language: language === 'auto' ? undefined : language,
        engine,
      });

      setActiveDraft(draft);
      syncResolutionsFromDraft(draft);
      addToast({
        type: 'success',
        title: t('stories.toast.extractedTitle', {
          count: draft.extracted_data.draft_people.length,
        }),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setExtracting(false);
    }
  };

  const handleSubmitClarification = async (clarificationText: string) => {
    if (!activeDraft || refining) return;
    setRefining(true);
    try {
      const updated = await storyApi.refineDraft(
        {
          draft_id: activeDraft.id,
          clarification_text: clarificationText,
        },
        activeDraft
      );
      setActiveDraft(updated);
      addToast({
        type: 'success',
        title: t('stories.toast.refinedTitle'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setRefining(false);
    }
  };

  const handleSavePersonInline = (updatedPerson: DraftPersonNodeDto) => {
    if (!activeDraft) return;
    setActiveDraft({
      ...activeDraft,
      extracted_data: {
        ...activeDraft.extracted_data,
        draft_people: activeDraft.extracted_data.draft_people.map((p) =>
          p.temp_id === updatedPerson.temp_id ? updatedPerson : p
        ),
      },
    });
  };

  const handleRemovePerson = (tempId: string) => {
    if (!activeDraft) return;
    setActiveDraft({
      ...activeDraft,
      extracted_data: {
        ...activeDraft.extracted_data,
        draft_people: activeDraft.extracted_data.draft_people.filter(
          (p) => p.temp_id !== tempId
        ),
      },
    });
    setResolutions((prev) => {
      const next = { ...prev };
      delete next[tempId];
      return next;
    });
  };

  const handleDiscardDraft = async () => {
    if (!activeDraft) return;
    const id = activeDraft.id;
    await storyApi.deleteDraft(id);
    setActiveDraft(null);
    setResolutions({});
    addToast({
      type: 'info',
      title: t('stories.toast.discardedTitle'),
    });
  };

  const handleCommitDraft = async () => {
    if (!activeDraft || committing) return;
    setCommitting(true);
    try {
      const res = await storyApi.commitDraft({
        draft_id: activeDraft.id,
        resolutions,
      });
      setActiveDraft(null);
      addToast({
        type: 'success',
        title: t('stories.toast.committedTitle', {
          newCount: res.created_count,
          linkedCount: res.linked_count,
        }),
        action: {
          label: t('history.undoAction'),
          onClick: () => {
            void historyApi
              .undoTransaction(res.tx_id)
              .catch(() => ({ status: 'undone' }))
              .then(() => {
                addToast({
                  type: 'info',
                  title: t('stories.toast.undoSuccessTitle'),
                });
              });
          },
        },
      });
      if (res.focal_person_id) {
        navigateTo(`/tree?focus=${encodeURIComponent(res.focal_person_id)}`);
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setCommitting(false);
    }
  };

  const currentStage = activeDraft ? 2 : 1;

  return (
    <div
      data-testid="ai-story-intake-page"
      className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6"
    >
      <PageHeader
        title={t('stories.pageTitle')}
        subtitle={t('stories.pageSubtitle')}
        secondaryActions={
          <Badge variant="brand" icon={<Sparkles className="h-3.5 w-3.5" />}>
            {t('stories.sandboxBadge')}
          </Badge>
        }
      />

      {/* 3-Stage Progression Tracker */}
      <Card
        data-testid="story-step-tracker"
        className="p-3.5 sm:p-4 flex flex-wrap items-center justify-between gap-3 bg-surface-subtle"
      >
        <div className="flex flex-wrap items-center gap-3 sm:gap-6 text-xs sm:text-sm">
          <div className="flex items-center gap-2 font-semibold text-primary">
            <Badge
              variant={currentStage >= 1 ? 'brand' : 'neutral'}
              size="sm"
            >
              1
            </Badge>
            <span>{t('stories.steps.step1')}</span>
          </div>

          <span className="text-tertiary">──</span>

          <div
            className={`flex items-center gap-2 ${
              currentStage >= 2
                ? 'font-semibold text-primary'
                : 'text-secondary'
            }`}
          >
            <Badge
              variant={currentStage >= 2 ? 'brand' : 'neutral'}
              size="sm"
            >
              2
            </Badge>
            <span>{t('stories.steps.step2')}</span>
          </div>

          <span className="text-tertiary">──</span>

          <div
            className={`flex items-center gap-2 ${
              currentStage >= 2
                ? 'font-semibold text-primary'
                : 'text-secondary'
            }`}
          >
            <Badge
              variant={currentStage >= 2 ? 'success' : 'neutral'}
              size="sm"
              icon={
                currentStage >= 2 ? (
                  <CheckCircle2 className="h-3 w-3" />
                ) : undefined
              }
            >
              3
            </Badge>
            <span>{t('stories.steps.step3')}</span>
          </div>
        </div>
      </Card>

      {/* Section 1: Speaker Context Selector */}
      <SpeakerSelectorCard
        speakers={speakers}
        selectedSpeakerId={selectedSpeakerId}
        onSelectSpeaker={setSelectedSpeakerId}
        isNewSpeaker={isNewSpeaker}
        onNewSpeakerChange={setIsNewSpeaker}
      />

      {/* Section 2: Multimodal Story Input Panel */}
      <StoryInputPanel
        narrativeText={narrativeText}
        onNarrativeTextChange={setNarrativeText}
        language={language}
        onLanguageChange={setLanguage}
        extracting={extracting}
        transcribing={transcribing}
        onExtractSubtree={() => void handleExtractSubtree()}
        onTranscribeBlob={(blob, speechTranscript) =>
          void handleTranscribeAudio(blob, speechTranscript)
        }
        engine={engine}
        onEngineChange={handleEngineChange}
      />

      {/* Loading Shimmer while Extracting */}
      {extracting && (
        <Card className="p-8 flex flex-col items-center justify-center gap-3 text-secondary">
          <Spinner size="lg" />
          <span className="text-sm font-medium">
            {t('stories.extractingLoadingText')}
          </span>
        </Card>
      )}

      {/* Section 3 & 4: Staged Subtree Workspace + Reconciliation Matrix */}
      {activeDraft && !extracting && (
        <div
          data-testid="staged-subtree-workspace"
          className="space-y-6"
        >
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
            <div className="lg:col-span-7 min-w-0">
              <DraftSubtreeCanvas
                people={activeDraft.extracted_data.draft_people}
                unions={activeDraft.extracted_data.draft_unions}
                parentChildLinks={
                  activeDraft.extracted_data.draft_parent_child
                }
                selectedTempId={editingPerson?.temp_id ?? null}
                onSelectPerson={setEditingPerson}
              />
            </div>

            <div className="lg:col-span-5 min-w-0">
              <DraftEntityListCard
                people={activeDraft.extracted_data.draft_people}
                unions={activeDraft.extracted_data.draft_unions}
                editingPerson={editingPerson}
                onEditPerson={setEditingPerson}
                onSavePerson={handleSavePersonInline}
                onRemovePerson={handleRemovePerson}
              />
            </div>
          </div>

          <DraftClarificationBar
            refining={refining}
            onSubmitClarification={(text) =>
              void handleSubmitClarification(text)
            }
          />

          <ReconciliationMatrixCard
            people={activeDraft.extracted_data.draft_people}
            candidates={activeDraft.extracted_data.reconciliation ?? []}
            resolutions={resolutions}
            onResolutionChange={(tempId, nextRes) =>
              setResolutions((prev) => ({
                ...prev,
                [tempId]: nextRes,
              }))
            }
            committing={committing}
            onCommit={() => void handleCommitDraft()}
            onDiscard={() => void handleDiscardDraft()}
          />
        </div>
      )}
    </div>
  );
};

export default AiStoryIntakePage;
