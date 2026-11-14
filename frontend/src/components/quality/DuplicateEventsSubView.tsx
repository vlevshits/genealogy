import React, { useMemo, useState } from 'react';
import { Calendar, Sparkles } from 'lucide-react';
import { Badge, Button, EmptyState, useToast } from '../ui';
import {
  ApiError,
  DuplicateEventGroupDto,
  MergeDuplicateEventsRequest,
  qualityApi,
} from '../../api/client';
import { useUndoToast } from '../../hooks/useUndoToast';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { DuplicateEventCard } from './DuplicateEventCard';
import { DuplicateEventCustomizeModal } from './DuplicateEventCustomizeModal';

export interface DuplicateEventsSubViewProps {
  groups: DuplicateEventGroupDto[];
  highlightedGroupId?: string | null;
  onMerged: (groupId: string) => void;
  onDismissed: (groupId: string) => void;
  onReload: () => void;
  onNavigate?: (path: string) => void;
}

export function isOfflineFallbackError(err: unknown): boolean {
  return (
    err instanceof ApiError &&
    (err.status === 0 || err.code === 'network_error')
  );
}

export function buildDefaultDuplicateEventGroups(
  locale: 'en' | 'ru' = 'en'
): DuplicateEventGroupDto[] {
  const isRu = locale === 'ru';
  return [
    {
      id: 'dup-evt-sample-1',
      event_type: 'BIRT',
      scope: 'individual',
      person: {
        id: 'sample-ivan-petrov',
        gid: 'I0001',
        display_name: isRu ? 'Иван Петров' : 'Ivan Petrov',
        sex: 'M',
        birth_date: '1938-05-12',
      },
      partner: null,
      events: [
        {
          id: 'ev-sample-1a',
          gid: 'E0101',
          event_type: 'BIRT',
          individual_id: 'sample-ivan-petrov',
          date_text: '1938-05-12',
          place_full_name: isRu
            ? 'Ярославль, Ярославская область'
            : 'Yaroslavl, Yaroslavl Oblast',
          description: isRu
            ? 'Метрическая запись о рождении'
            : 'Parish birth register entry',
          source_text: isRu ? 'ГАЯО, ф. 230' : 'Yaroslavl Archive, f. 230',
          media_count: 1,
          is_private: false,
        },
        {
          id: 'ev-sample-1b',
          gid: 'E0102',
          event_type: 'BIRT',
          individual_id: 'sample-ivan-petrov',
          date_text: '1938-05-12',
          place_full_name: isRu
            ? 'Ярославль, Ярославская область'
            : 'Yaroslavl, Yaroslavl Oblast',
          description: null,
          source_text: isRu ? 'Семейный архив' : 'Family archive',
          media_count: 1,
          is_private: false,
        },
      ],
      proposed: {
        keep_event_id: 'ev-sample-1a',
        keep_event_gid: 'E0101',
        event_type: 'BIRT',
        date_text: '1938-05-12',
        place_full_name: isRu
          ? 'Ярославль, Ярославская область'
          : 'Yaroslavl, Yaroslavl Oblast',
        description: isRu
          ? 'Метрическая запись о рождении'
          : 'Parish birth register entry',
        source_text: isRu
          ? 'ГАЯО, ф. 230 | Семейный архив'
          : 'Yaroslavl Archive, f. 230 | Family archive',
        combined_media_count: 2,
      },
      has_conflict: false,
      conflict_fields: [],
      confidence: 'high',
      reason: isRu
        ? 'Совпадающий тип, дата и место события для одной персоны'
        : 'Matching event type, date, and place on the same individual',
    },
    {
      id: 'dup-evt-sample-2',
      event_type: 'MARR',
      scope: 'family',
      person: {
        id: 'sample-ivan-petrov',
        gid: 'I0001',
        display_name: isRu ? 'Иван Петров' : 'Ivan Petrov',
        sex: 'M',
      },
      partner: {
        id: 'sample-elena-sokolova',
        gid: 'I0002',
        display_name: isRu ? 'Елена Соколова' : 'Elena Sokolova',
        sex: 'F',
      },
      family_ids: ['fam-sample-1', 'fam-sample-2'],
      events: [
        {
          id: 'ev-sample-2a',
          gid: 'E0201',
          event_type: 'MARR',
          family_id: 'fam-sample-1',
          date_text: '1962-06-14',
          place_full_name: isRu ? 'Кострома, Россия' : 'Kostroma, Russia',
          description: isRu ? 'ЗАГС № 1' : 'Civil registry #1',
          source_text: isRu
            ? 'Свидетельство о браке'
            : 'Marriage certificate',
          media_count: 1,
          is_private: false,
        },
        {
          id: 'ev-sample-2b',
          gid: 'E0202',
          event_type: 'MARR',
          family_id: 'fam-sample-2',
          date_text: '1962',
          place_full_name: isRu ? 'Кострома' : 'Kostroma',
          description: null,
          source_text: null,
          media_count: 0,
          is_private: false,
        },
      ],
      proposed: {
        keep_event_id: 'ev-sample-2a',
        keep_event_gid: 'E0201',
        event_type: 'MARR',
        date_text: '1962-06-14',
        place_full_name: isRu ? 'Кострома, Россия' : 'Kostroma, Russia',
        description: isRu ? 'ЗАГС № 1' : 'Civil registry #1',
        source_text: isRu
          ? 'Свидетельство о браке'
          : 'Marriage certificate',
        combined_media_count: 1,
      },
      has_conflict: true,
      conflict_fields: ['date', 'place'],
      confidence: 'medium',
      reason: isRu
        ? 'Дубликат записи о браке с разной точностью даты и места'
        : 'Same couple marriage event with differing date and place precision',
    },
  ];
}

export const DuplicateEventsSubView: React.FC<DuplicateEventsSubViewProps> = ({
  groups,
  highlightedGroupId = null,
  onMerged,
  onDismissed,
  onReload,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();
  const { showUndoToast } = useUndoToast();
  const [busyGroupId, setBusyGroupId] = useState<string | null>(null);
  const [batchMerging, setBatchMerging] = useState(false);
  const [customizingGroup, setCustomizingGroup] =
    useState<DuplicateEventGroupDto | null>(null);

  const autoMergeableGroups = useMemo(
    () => groups.filter((g) => !g.has_conflict && g.confidence === 'high'),
    [groups]
  );

  const executeMerge = async (
    group: DuplicateEventGroupDto,
    payload: MergeDuplicateEventsRequest
  ) => {
    setBusyGroupId(group.id);
    const keepEventGid =
      group.events.find((ev) => ev.id === payload.keep_event_id)?.gid ||
      group.proposed.keep_event_gid;

    try {
      const res = await qualityApi.mergeDuplicateEvents(payload);
      const survivingGid = res.surviving_event?.gid || keepEventGid;
      onMerged(group.id);
      setCustomizingGroup(null);

      if (res.tx_id) {
        showUndoToast({
          txId: res.tx_id,
          title: t('quality.duplicateEvents.mergedToastTitle'),
          description: t('quality.duplicateEvents.mergedToastDesc', {
            gid: survivingGid,
          }),
          onUndone: onReload,
        });
      } else {
        addToast({
          type: 'success',
          title: t('quality.duplicateEvents.mergedToastTitle'),
          description: t('quality.duplicateEvents.mergedToastDesc', {
            gid: survivingGid,
          }),
        });
      }
      onReload();
    } catch (err) {
      if (isOfflineFallbackError(err)) {
        onMerged(group.id);
        setCustomizingGroup(null);
        addToast({
          type: 'success',
          title: t('quality.duplicateEvents.mergedToastTitle'),
          description: t('quality.duplicateEvents.mergedToastDesc', {
            gid: keepEventGid,
          }),
        });
        return;
      }
      addToast({
        type: 'error',
        title: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setBusyGroupId(null);
    }
  };

  const handleQuickMerge = (group: DuplicateEventGroupDto) => {
    const payload: MergeDuplicateEventsRequest = {
      group_id: group.id,
      keep_event_id: group.proposed.keep_event_id,
      merge_event_ids: group.events
        .filter((ev) => ev.id !== group.proposed.keep_event_id)
        .map((ev) => ev.id),
      ...(group.proposed.date_text
        ? { date_text: group.proposed.date_text }
        : {}),
      ...(group.proposed.place_id
        ? { place_id: group.proposed.place_id }
        : {}),
      ...(group.proposed.place_full_name
        ? { place_name: group.proposed.place_full_name }
        : {}),
      ...(group.proposed.description
        ? { description: group.proposed.description }
        : {}),
      ...(group.proposed.source_text
        ? { source_text: group.proposed.source_text }
        : {}),
    };
    void executeMerge(group, payload);
  };

  const handleDismiss = async (group: DuplicateEventGroupDto) => {
    setBusyGroupId(group.id);
    try {
      await qualityApi
        .dismissWarning('DUPLICATE_EVENT', group.id)
        .catch((err) => {
          if (!isOfflineFallbackError(err)) throw err;
          return { status: 'dismissed' };
        });
      onDismissed(group.id);
      addToast({
        type: 'success',
        title: t('quality.duplicateEvents.dismissedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setBusyGroupId(null);
    }
  };

  const handleBatchMerge = async () => {
    if (autoMergeableGroups.length === 0) return;
    setBatchMerging(true);
    const targetIds = autoMergeableGroups.map((g) => g.id);

    try {
      const res = await qualityApi.batchMergeDuplicateEvents(targetIds, true);
      targetIds.forEach((id) => onMerged(id));

      const mergedCount = res.merged_groups_count || targetIds.length;
      if (res.tx_id) {
        showUndoToast({
          txId: res.tx_id,
          title: t('quality.duplicateEvents.batchMergedToastTitle', {
            count: mergedCount,
          }),
          description: t('quality.duplicateEvents.batchMergedToastDesc', {
            events: res.removed_events_count,
          }),
          onUndone: onReload,
        });
      } else {
        addToast({
          type: 'success',
          title: t('quality.duplicateEvents.batchMergedToastTitle', {
            count: mergedCount,
          }),
          description: t('quality.duplicateEvents.batchMergedToastDesc', {
            events: res.removed_events_count,
          }),
        });
      }
      onReload();
    } catch (err) {
      if (isOfflineFallbackError(err)) {
        targetIds.forEach((id) => onMerged(id));
        addToast({
          type: 'success',
          title: t('quality.duplicateEvents.batchMergedToastTitle', {
            count: targetIds.length,
          }),
        });
        return;
      }
      addToast({
        type: 'error',
        title: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setBatchMerging(false);
    }
  };

  return (
    <section
      data-testid="duplicate-events-section"
      aria-labelledby="duplicate-events-heading"
      className="space-y-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="min-w-0 max-w-full space-y-0.5">
          <div className="flex flex-wrap items-center gap-2">
            <Calendar className="h-4 w-4 shrink-0 text-brand" aria-hidden="true" />
            <h2
              id="duplicate-events-heading"
              className="text-base font-semibold text-primary"
            >
              {t('quality.duplicateEvents.sectionTitle')}
            </h2>
            <Badge variant="neutral">{groups.length}</Badge>
          </div>
          <p className="text-xs text-secondary">
            {t('quality.duplicateEvents.sectionSubtitle')}
          </p>
        </div>

        {autoMergeableGroups.length > 0 && (
          <Button
            variant="primary"
            size="md"
            loading={batchMerging}
            disabled={busyGroupId !== null}
            className="max-w-full"
            data-testid="batch-merge-events-btn"
            leftIcon={<Sparkles className="h-4 w-4" />}
            onClick={() => void handleBatchMerge()}
          >
            {t('quality.duplicateEvents.autoMergeAllBtn', {
              count: autoMergeableGroups.length,
            })}
          </Button>
        )}
      </div>

      {groups.length === 0 ? (
        <EmptyState
          icon={<Calendar className="h-8 w-8" />}
          title={t('quality.duplicateEvents.emptyTitle')}
          description={t('quality.duplicateEvents.emptyDesc')}
        />
      ) : (
        <div className="space-y-3">
          {groups.map((group) => (
            <DuplicateEventCard
              key={group.id}
              group={group}
              busy={busyGroupId === group.id || batchMerging}
              highlighted={highlightedGroupId === group.id}
              onNavigate={onNavigate}
              onDismiss={(g) => void handleDismiss(g)}
              onCustomize={setCustomizingGroup}
              onMerge={handleQuickMerge}
            />
          ))}
        </div>
      )}

      <DuplicateEventCustomizeModal
        open={Boolean(customizingGroup)}
        onOpenChange={(open) => {
          if (!open) setCustomizingGroup(null);
        }}
        group={customizingGroup}
        saving={
          Boolean(customizingGroup) && busyGroupId === customizingGroup?.id
        }
        onSubmit={(payload) => {
          if (customizingGroup) {
            void executeMerge(customizingGroup, payload);
          }
        }}
      />
    </section>
  );
};
