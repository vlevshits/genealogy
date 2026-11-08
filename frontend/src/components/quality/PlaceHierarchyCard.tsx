import React, { useState } from 'react';
import {
  CheckCircle2,
  CornerDownRight,
  MapPin,
  SlidersHorizontal,
  Sparkles,
  XCircle,
} from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import { PlaceHierarchyIssueDto } from '../../api/client';
import { useI18n } from '../../i18n';
import { normalizePlaceType } from '../../lib/domain-enums';

export interface PlaceHierarchyCardProps {
  issue: PlaceHierarchyIssueDto;
  fixing?: boolean;
  onApplyFix: (issue: PlaceHierarchyIssueDto, selectedParentId?: string | null) => void;
  onCustomize: (issue: PlaceHierarchyIssueDto) => void;
  onDismiss: (issue: PlaceHierarchyIssueDto) => void;
}

export const PlaceHierarchyCard: React.FC<PlaceHierarchyCardProps> = ({
  issue,
  fixing = false,
  onApplyFix,
  onCustomize,
  onDismiss,
}) => {
  const { t } = useI18n();
  const [selectedCandidateId, setSelectedCandidateId] = useState<string | null>(
    issue.proposed_parent_id ??
      issue.candidate_parents?.[0]?.place_id ??
      null
  );

  const rawScore = issue.confidence_score ?? issue.confidence ?? 85;
  const numericScore = Number.isFinite(rawScore)
    ? rawScore > 0 && rawScore <= 1
      ? rawScore * 100
      : rawScore
    : 85;
  const scorePct = Math.round(numericScore);

  const tier =
    issue.confidence_tier === 'high' ||
    issue.confidence_tier === 'medium' ||
    issue.confidence_tier === 'low'
      ? issue.confidence_tier
      : scorePct >= 85
        ? 'high'
        : scorePct >= 60
          ? 'medium'
          : 'low';

  const tierVariant =
    tier === 'high'
      ? 'success'
      : tier === 'medium'
        ? 'warning'
        : 'neutral';

  const getCategoryLabel = (cat: string): string => {
    switch (cat) {
      case 'unparsed_path':
      case 'unparsed_paths':
        return t('quality.places.category.unparsed');
      case 'orphan_hierarchy':
      case 'orphans':
        return t('quality.places.category.orphans');
      case 'type_mismatch':
      case 'type_mismatches':
        return t('quality.places.category.typeMismatch');
      case 'duplicate_node':
      case 'duplicates':
        return t('quality.places.category.duplicates');
      default:
        if (issue.rule_code === 'PLACE_UNPARSED_PATH') {
          return t('quality.places.category.unparsed');
        }
        if (issue.rule_code === 'PLACE_ORPHAN_HIERARCHY') {
          return t('quality.places.category.orphans');
        }
        if (issue.rule_code === 'PLACE_TYPE_MISMATCH') {
          return t('quality.places.category.typeMismatch');
        }
        if (issue.rule_code === 'PLACE_DUPLICATE_CANDIDATE') {
          return t('quality.places.category.duplicates');
        }
        return cat;
    }
  };

  const getStepActionBadge = (action: string) => {
    if (action === 'reuse' || action === 'match_existing') {
      return (
        <Badge variant="neutral" size="sm">
          {t('quality.places.stepAction.reuse')}
        </Badge>
      );
    }
    if (action === 'create' || action === 'create_new') {
      return (
        <Badge variant="brand" size="sm">
          {t('quality.places.stepAction.create')}
        </Badge>
      );
    }
    if (action === 'consolidate') {
      return (
        <Badge variant="warning" size="sm">
          {t('quality.places.stepAction.consolidate')}
        </Badge>
      );
    }
    return (
      <Badge variant="success" size="sm">
        {t('quality.places.stepAction.cleanLeaf')}
      </Badge>
    );
  };

  const visibleChain = (issue.proposed_chain ?? []).filter((step) => {
    const lower = (step.name || '').trim().toLowerCase();
    return Boolean(lower) && lower !== 'unknown' && lower !== 'неизвестно';
  });

  const visibleCandidates = (
    issue.candidate_parents ??
    issue.parent_candidates ??
    []
  ).filter((cand) => {
    const lower = (cand.name || '').trim().toLowerCase();
    return Boolean(lower) && lower !== 'unknown' && lower !== 'неизвестно';
  });

  return (
    <Card
      data-testid={`place-hierarchy-card-${issue.place_id}`}
      className="p-4 sm:p-5 space-y-4"
    >
      {/* Card Header */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={tierVariant} size="sm">
            {t(`quality.places.confidence.${tier}`, {
              score: scorePct,
              pct: scorePct,
            })}
          </Badge>
          <Badge variant="neutral" size="sm">
            {getCategoryLabel(issue.category || issue.issue_type || '')}
          </Badge>
          <span className="font-mono text-xs text-tertiary">
            ({issue.place_gid})
          </span>
        </div>
        <span className="text-xs text-secondary">
          {t('quality.places.eventsCount', { count: issue.events_count })}
        </span>
      </div>

      {/* Current vs Proposed Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 rounded-xl bg-surface-muted p-3.5 border border-border">
        {/* Current Place */}
        <div className="space-y-1.5 min-w-0">
          <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
            {t('quality.places.currentRecordLabel')}
          </div>
          <div className="flex items-start gap-2">
            <MapPin className="h-4 w-4 text-warning shrink-0 mt-0.5" />
            <div className="min-w-0">
              <div className="text-sm font-semibold text-primary break-words">
                {issue.current_name}
              </div>
              <div className="text-xs text-secondary mt-0.5 flex flex-wrap items-center gap-1.5">
                <span>
                  {t('quality.places.typeLabel')}:{' '}
                  {t(`places.types.${normalizePlaceType(issue.current_place_type)}`)}
                </span>
                <span>·</span>
                <span>
                  {t('quality.places.parentLabel')}:{' '}
                  {issue.current_parent_name || t('quality.places.noParent')}
                </span>
                {issue.latitude != null && issue.longitude != null && (
                  <>
                    <span>·</span>
                    <span className="font-mono">
                      {issue.latitude.toFixed(2)}, {issue.longitude.toFixed(2)}
                    </span>
                  </>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Proposed Hierarchy Chain */}
        <div className="space-y-1.5 min-w-0">
          <div className="text-xs font-semibold uppercase tracking-wider text-tertiary">
            {t('quality.places.proposedHierarchyLabel')}
          </div>
          {visibleChain.length > 0 ? (
            <div className="space-y-1.5">
              {visibleChain.map((step, idx) => {
                const isLast = idx === visibleChain.length - 1;
                const rawCategory = issue.category || issue.issue_type || '';
                const resolvedAction =
                  step.action ||
                  (isLast
                    ? issue.consolidate_with_place_id ||
                      rawCategory === 'duplicates' ||
                      rawCategory === 'duplicate_node' ||
                      issue.rule_code === 'PLACE_DUPLICATE_CANDIDATE'
                      ? 'consolidate'
                      : 'clean_leaf'
                    : step.is_new || !step.existing_place_id
                      ? 'create'
                      : 'reuse');
                return (
                  <div
                    key={`${step.name}-${idx}`}
                    className="flex flex-wrap items-center gap-1.5 text-xs"
                    style={{ paddingLeft: `${idx * 14}px` }}
                  >
                    {idx > 0 && (
                      <CornerDownRight className="h-3.5 w-3.5 text-tertiary shrink-0" />
                    )}
                    <span className="font-medium text-primary">{step.name}</span>
                    <span className="text-tertiary">
                      ({t(`places.types.${normalizePlaceType(step.place_type)}`)})
                    </span>
                    {getStepActionBadge(resolvedAction)}
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="text-xs text-secondary">
              {issue.proposed_clean_name} (
              {t(`places.types.${normalizePlaceType(issue.proposed_place_type)}`)})
            </div>
          )}

          {/* Candidate Parents Selector for Medium/Ambiguous Issues */}
          {visibleCandidates.length > 1 && (
            <div className="pt-2 space-y-1.5 border-t border-border mt-2">
              <div className="text-xs font-medium text-secondary">
                {t('quality.places.selectCandidateParentLabel')}:
              </div>
              <div className="flex flex-wrap gap-1.5">
                {visibleCandidates.map((cand) => {
                  const isSelected = selectedCandidateId === cand.place_id;
                  return (
                    <Button
                      key={cand.place_id}
                      size="sm"
                      variant={isSelected ? 'primary' : 'secondary'}
                      onClick={() => setSelectedCandidateId(cand.place_id)}
                    >
                      {cand.full_name}
                      {cand.distance_km != null
                        ? ` (${Math.round(cand.distance_km)} km)`
                        : ''}
                    </Button>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Rule Rationale & Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-1">
        <p className="text-xs text-secondary">
          {issue.rationale || issue.reason || ''}
        </p>

        <div className="flex flex-wrap items-center gap-2 shrink-0">
          <Button
            size="sm"
            variant="ghost"
            data-testid={`dismiss-place-issue-btn-${issue.place_id}`}
            leftIcon={<XCircle className="h-3.5 w-3.5" />}
            onClick={() => onDismiss(issue)}
          >
            {t('quality.dismissBtn')}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            data-testid={`customize-place-issue-btn-${issue.place_id}`}
            leftIcon={<SlidersHorizontal className="h-3.5 w-3.5" />}
            onClick={() => onCustomize(issue)}
          >
            {t('quality.places.customizeBtn')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            loading={fixing}
            data-testid={`apply-place-fix-btn-${issue.place_id}`}
            leftIcon={
              issue.confidence_tier === 'high' ? (
                <Sparkles className="h-3.5 w-3.5" />
              ) : (
                <CheckCircle2 className="h-3.5 w-3.5" />
              )
            }
            onClick={() => onApplyFix(issue, selectedCandidateId)}
          >
            {t('quality.places.applyFixBtn')}
          </Button>
        </div>
      </div>
    </Card>
  );
};
