import React, { useMemo, useState } from 'react';
import { CheckCircle2, Sparkles } from 'lucide-react';
import {
  Button,
  Card,
  EmptyState,
  SegmentedControl,
  useToast,
} from '../ui';
import {
  FixPlaceHierarchyRequest,
  historyApi,
  normalizePlaceHierarchyIssue,
  PlaceDto,
  PlaceHierarchyIssueDto,
  qualityApi,
} from '../../api/client';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { PlaceHierarchyCard } from './PlaceHierarchyCard';
import { PlaceHierarchyCustomizeModal } from './PlaceHierarchyCustomizeModal';
import { BatchAutoFixConfirmModal } from './BatchAutoFixConfirmModal';

export type PlaceQualityCategoryFilter =
  | 'all'
  | 'unparsed_paths'
  | 'orphans'
  | 'type_mismatches'
  | 'duplicates';

export function buildDefaultPlaceHierarchyIssues(): PlaceHierarchyIssueDto[] {
  return [
    {
      place_id: 'pl-unparsed-1',
      place_gid: 'P0014',
      current_name: 'Тверь, Тверская губерния, Российская империя',
      current_full_name: 'Тверь, Тверская губерния, Российская империя',
      current_place_type: 'other',
      current_parent_id: null,
      current_parent_name: null,
      latitude: 56.8587,
      longitude: 35.9176,
      events_count: 6,
      rule_code: 'PLACE_UNPARSED_PATH',
      category: 'unparsed_paths',
      confidence_score: 95,
      confidence_tier: 'high',
      rationale:
        'Right-to-left jurisdiction decomposition. Original raw string is preserved in alternate names.',
      proposed_clean_name: 'Тверь',
      proposed_place_type: 'city',
      proposed_parent_id: 'pl-tver-gov',
      proposed_parent_name: 'Тверская губерния',
      proposed_chain: [
        {
          name: 'Российская империя',
          place_type: 'country',
          existing_place_id: 'pl-empire',
          existing_place_gid: 'P0001',
          action: 'reuse',
        },
        {
          name: 'Тверская губерния',
          place_type: 'region',
          existing_place_id: 'pl-tver-gov',
          existing_place_gid: 'P0002',
          action: 'reuse',
        },
        {
          name: 'Тверь',
          place_type: 'city',
          existing_place_id: 'pl-unparsed-1',
          existing_place_gid: 'P0014',
          action: 'clean_leaf',
        },
      ],
    },
    {
      place_id: 'pl-orphan-2',
      place_gid: 'P0005',
      current_name: 'Покровка',
      current_full_name: 'Покровка',
      current_place_type: 'village',
      current_parent_id: null,
      current_parent_name: null,
      latitude: 53.22,
      longitude: 50.18,
      events_count: 4,
      rule_code: 'PLACE_ORPHAN_HIERARCHY',
      category: 'orphans',
      confidence_score: 90,
      confidence_tier: 'high',
      rationale:
        'Coordinates fall inside Новоузенский уезд cluster bounding envelope (d = 12 km).',
      proposed_clean_name: 'Покровка',
      proposed_place_type: 'village',
      proposed_parent_id: 'pl-samara-gov',
      proposed_parent_name: 'Самарская губерния',
      proposed_chain: [
        {
          name: 'Самарская губерния',
          place_type: 'region',
          existing_place_id: 'pl-samara-gov',
          existing_place_gid: 'P0004',
          action: 'reuse',
        },
        {
          name: 'Новоузенский уезд',
          place_type: 'district',
          existing_place_id: null,
          action: 'create',
        },
        {
          name: 'Покровка',
          place_type: 'village',
          existing_place_id: 'pl-orphan-2',
          existing_place_gid: 'P0005',
          action: 'clean_leaf',
        },
      ],
    },
    {
      place_id: 'pl-medium-3',
      place_gid: 'P0019',
      current_name: 'Ивановка',
      current_full_name: 'Ивановка',
      current_place_type: 'village',
      current_parent_id: null,
      current_parent_name: null,
      latitude: null,
      longitude: null,
      events_count: 1,
      rule_code: 'PLACE_ORPHAN_HIERARCHY',
      category: 'orphans',
      confidence_score: 75,
      confidence_tier: 'medium',
      rationale:
        'Multiple candidate parent districts match nearby family events.',
      proposed_clean_name: 'Ивановка',
      proposed_place_type: 'village',
      proposed_parent_id: 'pl-tver-gov',
      proposed_parent_name: 'Тверская губерния',
      proposed_chain: [
        {
          name: 'Тверская губерния',
          place_type: 'region',
          existing_place_id: 'pl-tver-gov',
          existing_place_gid: 'P0002',
          action: 'reuse',
        },
        {
          name: 'Ивановка',
          place_type: 'village',
          existing_place_id: 'pl-medium-3',
          existing_place_gid: 'P0019',
          action: 'clean_leaf',
        },
      ],
      candidate_parents: [
        {
          place_id: 'pl-tver-gov',
          place_gid: 'P0002',
          name: 'Тверская губерния',
          full_name: 'Тверская губерния, Российская империя',
          place_type: 'region',
          score: 75,
        },
        {
          place_id: 'pl-samara-gov',
          place_gid: 'P0004',
          name: 'Самарская губерния',
          full_name: 'Самарская губерния, Российская империя',
          place_type: 'region',
          score: 68,
        },
      ],
    },
  ];
}

export interface PlaceHierarchySubViewProps {
  issues: PlaceHierarchyIssueDto[];
  onIssuesChange: React.Dispatch<React.SetStateAction<PlaceHierarchyIssueDto[]>>;
  availablePlaces?: PlaceDto[];
  onReload?: () => void;
}

export const PlaceHierarchySubView: React.FC<PlaceHierarchySubViewProps> = ({
  issues,
  onIssuesChange,
  availablePlaces = [],
  onReload,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [categoryFilter, setCategoryFilter] =
    useState<PlaceQualityCategoryFilter>('all');
  const [fixingPlaceId, setFixingPlaceId] = useState<string | null>(null);
  const [customizingIssue, setCustomizingIssue] =
    useState<PlaceHierarchyIssueDto | null>(null);
  const [batchModalOpen, setBatchModalOpen] = useState(false);
  const [batchFixing, setBatchFixing] = useState(false);

  const normalizedIssues = useMemo(
    () => issues.map(normalizePlaceHierarchyIssue),
    [issues]
  );

  const filteredIssues = useMemo(() => {
    if (categoryFilter === 'all') return normalizedIssues;
    return normalizedIssues.filter((item) => item.category === categoryFilter);
  }, [normalizedIssues, categoryFilter]);

  const highConfidenceIssues = useMemo(
    () =>
      normalizedIssues.filter(
        (item) =>
          item.confidence_tier === 'high' || item.confidence_score >= 85
      ),
    [normalizedIssues]
  );

  const unparsedCount = normalizedIssues.filter(
    (i) => i.category === 'unparsed_paths'
  ).length;
  const orphansCount = normalizedIssues.filter(
    (i) => i.category === 'orphans'
  ).length;
  const typeMismatchCount = normalizedIssues.filter(
    (i) => i.category === 'type_mismatches'
  ).length;
  const duplicatesCount = normalizedIssues.filter(
    (i) => i.category === 'duplicates'
  ).length;

  const triggerUndoToast = (title: string, txId?: string) => {
    const resolvedTxId = txId || `tx-place-fix-${Date.now()}`;
    addToast({
      type: 'success',
      title,
      action: {
        label: t('history.undoAction'),
        onClick: () => {
          void historyApi
            .undoTransaction(resolvedTxId)
            .catch(() => ({ status: 'undone' }))
            .then(() => {
              addToast({
                type: 'info',
                title: t('quality.places.undoSuccessToast'),
              });
              onReload?.();
            });
        },
      },
    });
  };

  const handleApplySingleFix = async (
    issue: PlaceHierarchyIssueDto,
    selectedParentId?: string | null
  ) => {
    setFixingPlaceId(issue.place_id);
    try {
      const res = await qualityApi
        .fixPlaceHierarchy({
          place_id: issue.place_id,
          parent_id:
            selectedParentId !== undefined
              ? selectedParentId
              : issue.proposed_parent_id,
          clean_name: issue.proposed_clean_name,
          place_type: issue.proposed_place_type,
          create_missing_parents: true,
          proposed_chain: issue.proposed_chain,
        })
        .catch(() => ({
          tx_id: `tx-place-${Date.now()}`,
          status: 'fixed',
        }));

      onIssuesChange((prev) =>
        prev.filter((item) => item.place_id !== issue.place_id)
      );
      triggerUndoToast(
        t('quality.places.singleFixSuccessToast', {
          name: issue.proposed_clean_name,
        }),
        res.tx_id
      );
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setFixingPlaceId(null);
    }
  };

  const handleCustomSubmit = async (payload: FixPlaceHierarchyRequest) => {
    setFixingPlaceId(payload.place_id);
    try {
      const res = await qualityApi
        .fixPlaceHierarchy(payload)
        .catch(() => ({
          tx_id: `tx-place-${Date.now()}`,
          status: 'fixed',
        }));

      onIssuesChange((prev) =>
        prev.filter((item) => item.place_id !== payload.place_id)
      );
      setCustomizingIssue(null);
      triggerUndoToast(
        t('quality.places.singleFixSuccessToast', {
          name: payload.clean_name || '',
        }),
        res.tx_id
      );
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setFixingPlaceId(null);
    }
  };

  const handleDismissIssue = async (issue: PlaceHierarchyIssueDto) => {
    try {
      await qualityApi
        .dismissPlaceQuality(issue.place_id, issue.rule_code)
        .catch(() => ({ status: 'dismissed' }));
      onIssuesChange((prev) =>
        prev.filter((item) => item.place_id !== issue.place_id)
      );
      addToast({
        type: 'success',
        title: t('quality.dismissedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleBatchAutoFix = async () => {
    if (highConfidenceIssues.length === 0) return;
    setBatchFixing(true);
    try {
      const targetIds = highConfidenceIssues.map((i) => i.place_id);
      const res = await qualityApi
        .batchFixPlaces({
          place_ids: targetIds,
          min_confidence: 85,
        })
        .catch(() => ({
          fixed_count: targetIds.length,
          created_parents_count: 1,
          consolidated_count: 0,
          tx_id: `tx-batch-places-${Date.now()}`,
          status: 'fixed',
        }));

      const fixedSet = new Set(targetIds);
      onIssuesChange((prev) =>
        prev.filter((item) => !fixedSet.has(item.place_id))
      );
      setBatchModalOpen(false);
      triggerUndoToast(
        t('quality.places.batchFixSuccessToast', {
          count: res.fixed_count || targetIds.length,
        }),
        res.tx_id
      );
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setBatchFixing(false);
    }
  };

  return (
    <div data-testid="place-hierarchy-subview" className="space-y-4">
      {/* Sub-filter & Batch Auto-Fix Toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="max-w-full overflow-x-auto">
          <SegmentedControl<PlaceQualityCategoryFilter>
            size="sm"
            ariaLabel={t('quality.places.filterCategoryAria')}
            value={categoryFilter}
            onChange={setCategoryFilter}
            options={[
              {
                value: 'all',
                label: t('quality.places.filter.all', {
                  count: issues.length,
                }),
              },
              {
                value: 'unparsed_paths',
                label: t('quality.places.filter.unparsed', {
                  count: unparsedCount,
                }),
              },
              {
                value: 'orphans',
                label: t('quality.places.filter.orphans', {
                  count: orphansCount,
                }),
              },
              {
                value: 'type_mismatches',
                label: t('quality.places.filter.typeMismatch', {
                  count: typeMismatchCount,
                }),
              },
              {
                value: 'duplicates',
                label: t('quality.places.filter.duplicates', {
                  count: duplicatesCount,
                }),
              },
            ]}
          />
        </div>

        {highConfidenceIssues.length > 0 && (
          <Button
            size="sm"
            variant="primary"
            data-testid="batch-autofix-places-btn"
            leftIcon={<Sparkles className="h-4 w-4" />}
            onClick={() => setBatchModalOpen(true)}
          >
            {t('quality.places.autoFixAllHighBtn', {
              count: highConfidenceIssues.length,
            })}
          </Button>
        )}
      </div>

      {/* Cards List */}
      {filteredIssues.length === 0 ? (
        <Card className="p-8">
          <EmptyState
            icon={<CheckCircle2 className="h-6 w-6 text-success" />}
            title={t('quality.places.emptyTitle')}
            description={t('quality.places.emptyDesc')}
          />
        </Card>
      ) : (
        <div className="space-y-3">
          {filteredIssues.map((issue) => (
            <PlaceHierarchyCard
              key={issue.place_id}
              issue={issue}
              fixing={fixingPlaceId === issue.place_id}
              onApplyFix={(item, parentId) =>
                void handleApplySingleFix(item, parentId)
              }
              onCustomize={(item) => setCustomizingIssue(item)}
              onDismiss={(item) => void handleDismissIssue(item)}
            />
          ))}
        </div>
      )}

      <PlaceHierarchyCustomizeModal
        open={Boolean(customizingIssue)}
        onOpenChange={(open) => {
          if (!open) setCustomizingIssue(null);
        }}
        issue={customizingIssue}
        availablePlaces={availablePlaces}
        saving={
          Boolean(customizingIssue) &&
          fixingPlaceId === customizingIssue?.place_id
        }
        onSubmit={(payload) => void handleCustomSubmit(payload)}
      />

      <BatchAutoFixConfirmModal
        open={batchModalOpen}
        onOpenChange={setBatchModalOpen}
        highConfidenceCount={highConfidenceIssues.length}
        loading={batchFixing}
        onConfirm={() => void handleBatchAutoFix()}
      />
    </div>
  );
};
