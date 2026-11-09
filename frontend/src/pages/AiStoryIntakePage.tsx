import React, { useCallback, useEffect, useState } from 'react';
import { CheckCircle2, GitMerge, Sparkles } from 'lucide-react';
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
import { PeopleDirectoryItemDto } from '../api/openapi-types-domain';
import { Individual } from '../types/person';
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

const toIndividualFromDirectory = (item: PeopleDirectoryItemDto): Individual => ({
  id: item.id,
  gid: item.gid,
  display_name: item.display_name,
  surname: item.surname,
  given_name: item.given_name,
  patronymic: item.patronymic,
  birth_surname: item.birth_surname,
  sex:
    item.sex === 'M' || item.sex === 'male'
      ? 'male'
      : item.sex === 'F' || item.sex === 'female'
        ? 'female'
        : 'unknown',
  is_living: item.is_living,
  is_private: false,
  avatar_url: item.avatar_url,
  birth_place: item.birth_place,
  version: 1,
});

const toIndividualFromSpeakerOption = (sp: SpeakerOption): Individual => ({
  id: sp.id,
  gid: sp.gid,
  display_name: sp.display_name,
  given_name: sp.display_name.split(' ')[0] || sp.display_name,
  sex: 'unknown',
  is_living: true,
  is_private: false,
  version: 1,
});

const DEFAULT_SPEAKERS_RU: SpeakerOption[] = [
  { id: 'node-root', gid: 'I0001', display_name: 'Анна Сергеевна Волкова' },
  { id: 'node-mother-1', gid: 'I0002', display_name: 'Мария Ивановна Волкова' },
  { id: 'node-deceased-1', gid: 'I0012', display_name: 'Пётр Фёдорович Щеглов' },
];

const DEFAULT_SPEAKERS_EN: SpeakerOption[] = [
  { id: 'node-root', gid: 'I0001', display_name: 'Anna Sergeyevna Volkova' },
  { id: 'node-mother-1', gid: 'I0002', display_name: 'Maria Ivanovna Volkova' },
  { id: 'node-deceased-1', gid: 'I0012', display_name: 'Pyotr Fyodorovich Shcheglov' },
];

export const AiStoryIntakePage: React.FC<AiStoryIntakePageProps> = ({
  initialDraft = null,
  initialSpeakers,
  userRole,
  onNavigate,
}) => {
  const { t, locale } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const canEdit = effectiveRole === 'admin' || effectiveRole === 'editor';

  const defaultSpeakers =
    locale === 'ru' ? DEFAULT_SPEAKERS_RU : DEFAULT_SPEAKERS_EN;

  const [speakers, setSpeakers] = useState<SpeakerOption[]>(
    initialSpeakers ?? defaultSpeakers
  );
  const [people, setPeople] = useState<Individual[]>(() =>
    (initialSpeakers ?? defaultSpeakers).map(toIndividualFromSpeakerOption)
  );
  const [selectedSpeakerId, setSelectedSpeakerId] = useState<string>(
    user?.person_id || user?.home_person_id || defaultSpeakers[0].id
  );
  const [isNewSpeaker, setIsNewSpeaker] = useState<boolean>(false);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loadingMore, setLoadingMore] = useState<boolean>(false);
  const [currentSearchQuery, setCurrentSearchQuery] = useState<string>('');
  const [speakerError, setSpeakerError] = useState<string | null>(null);

  const [narrativeText, setNarrativeText] = useState<string>(
    initialDraft?.raw_text ?? ''
  );
  const [language, setLanguage] = useState<string>('auto');
  const [transcribing, setTranscribing] = useState<boolean>(false);
  const [extracting, setExtracting] = useState<boolean>(false);
  const [refining, setRefining] = useState<boolean>(false);
  const [committing, setCommitting] = useState<boolean>(false);
  const [isCommitted, setIsCommitted] = useState<boolean>(false);

  const [engine, setEngine] = useState<StoryExtractionEngine>(() =>
    getExtractionEngine()
  );

  useEffect(() => {
    if (initialSpeakers) return;
    setSpeakers((prev) => {
      const isDefaultList = prev.every(
        (s) =>
          s.id === 'node-root' ||
          s.id === 'node-mother-1' ||
          s.id === 'node-deceased-1'
      );
      if (isDefaultList) {
        const nextDefaults =
          locale === 'ru' ? DEFAULT_SPEAKERS_RU : DEFAULT_SPEAKERS_EN;
        setPeople(nextDefaults.map(toIndividualFromSpeakerOption));
        return nextDefaults;
      }
      return prev;
    });
  }, [locale, initialSpeakers]);

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
          const mapped = res.items.map(toIndividualFromDirectory);
          setPeople(mapped);
          setSpeakers(
            res.items.map((item) => ({
              id: item.id,
              gid: item.gid,
              display_name: item.display_name,
            }))
          );
          setNextCursor(res.next_cursor);

          setSelectedSpeakerId((prev) => {
            if (prev === 'node-root') {
              if (user?.person_id && res.items.some((p) => p.id === user.person_id)) {
                return user.person_id;
              }
              if (user?.home_person_id && res.items.some((p) => p.id === user.home_person_id)) {
                return user.home_person_id;
              }
              return '';
            }
            return prev;
          });
        }
      })
      .catch(() => { });
    return () => {
      cancelled = true;
    };
  }, [canEdit, initialSpeakers, user?.person_id, user?.home_person_id]);

  const loadPeopleForSpeaker = useCallback(
    async (query: string, signal: AbortSignal): Promise<Individual[]> => {
      setCurrentSearchQuery(query);
      try {
        const res = await fetchPeopleDirectory({
          q: query.trim() || undefined,
          limit: 30,
        });
        if (signal.aborted) return [];
        setNextCursor(res.next_cursor);
        const mapped = res.items.map(toIndividualFromDirectory);
        setPeople((prev) => {
          const ids = new Set(mapped.map((p) => p.id));
          return [...mapped, ...prev.filter((p) => !ids.has(p.id))];
        });
        return mapped;
      } catch {
        return [];
      }
    },
    []
  );

  const handleLoadMoreSpeakers = useCallback(async () => {
    if (!nextCursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const res = await fetchPeopleDirectory({
        q: currentSearchQuery.trim() || undefined,
        cursor: nextCursor,
        limit: 30,
      });
      setNextCursor(res.next_cursor);
      const mapped = res.items.map(toIndividualFromDirectory);
      setPeople((prev) => {
        const ids = new Set(mapped.map((p) => p.id));
        const combined = [...prev];
        for (const p of mapped) {
          if (!ids.has(p.id)) {
            combined.push(p);
            ids.add(p.id);
          }
        }
        return combined;
      });
    } catch {
      // Ignore load more errors
    } finally {
      setLoadingMore(false);
    }
  }, [nextCursor, loadingMore, currentSearchQuery]);

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
    const trimmedText = narrativeText.trim();
    if (!trimmedText || extracting) return;

    if (!isNewSpeaker && (!selectedSpeakerId || !selectedSpeakerId.trim())) {
      const msg = t('stories.speaker.requiredError');
      setSpeakerError(msg);
      addToast({
        type: 'error',
        title: t('stories.speaker.requiredErrorTitle') || t('common.errorLoading'),
        description: msg,
      });
      return;
    }

    setExtracting(true);
    setSpeakerError(null);

    try {
      const draft = await storyApi.parseStory({
        speaker_person_id: isNewSpeaker ? null : selectedSpeakerId.trim(),
        is_new_speaker: isNewSpeaker,
        raw_text: trimmedText,
        language: language === 'auto' ? undefined : language,
        engine,
      });

      setIsCommitted(false);
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
    setIsCommitted(false);
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
      setIsCommitted(true);
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

  const currentStage = isCommitted ? 3 : activeDraft ? 2 : 1;

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
        className="p-3.5 sm:p-4 flex flex-wrap items-center justify-between gap-3 bg-surface-muted"
      >
        <div className="flex flex-wrap items-center gap-3 sm:gap-6 text-xs sm:text-sm">
          <div className="flex items-center gap-2 font-semibold text-primary">
            <Badge
              variant={currentStage > 1 ? 'success' : 'brand'}
              size="sm"
              icon={
                currentStage > 1 ? (
                  <CheckCircle2 className="h-3 w-3" />
                ) : undefined
              }
            >
              1
            </Badge>
            <span>{t('stories.steps.step1')}</span>
          </div>

          <span className="text-tertiary">──</span>

          <div
            className={`flex items-center gap-2 ${currentStage >= 2
                ? 'font-semibold text-primary'
                : 'text-secondary'
              }`}
          >
            <Badge
              variant={
                currentStage > 2
                  ? 'success'
                  : currentStage === 2
                    ? 'brand'
                    : 'neutral'
              }
              size="sm"
              icon={
                currentStage > 2 ? (
                  <CheckCircle2 className="h-3 w-3" />
                ) : undefined
              }
            >
              2
            </Badge>
            <span>{t('stories.steps.step2')}</span>
          </div>

          <span className="text-tertiary">──</span>

          <div
            className={`flex items-center gap-2 ${currentStage === 3
                ? 'font-semibold text-primary'
                : 'text-secondary'
              }`}
          >
            <Badge
              variant={currentStage === 3 ? 'success' : 'neutral'}
              size="sm"
              icon={
                currentStage === 3 ? (
                  <CheckCircle2 className="h-3 w-3" />
                ) : (
                  <GitMerge className="h-3 w-3" />
                )
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
        people={people}
        selectedSpeakerId={selectedSpeakerId}
        onSelectSpeaker={(id) => {
          setSelectedSpeakerId(id || '');
          setSpeakerError(null);
        }}
        isNewSpeaker={isNewSpeaker}
        onNewSpeakerChange={(isNew) => {
          setIsNewSpeaker(isNew);
          setSpeakerError(null);
        }}
        loadPeople={loadPeopleForSpeaker}
        hasMore={Boolean(nextCursor)}
        loadingMore={loadingMore}
        onLoadMore={() => void handleLoadMoreSpeakers()}
        errorMessage={speakerError}
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
