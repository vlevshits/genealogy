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

  const scorePct =
    issue.confidence_score <= 1
      ? Math.round(issue.confidence_score * 100)
      : Math.round(issue.confidence_score);

  const tierVariant =
    issue.confidence_tier === 'high'
      ? 'success'
      : issue.confidence_tier === 'medium'
        ? 'warning'
        : 'neutral';

  const getCategoryLabel = (cat: string): string => {
    switch (cat) {
      case 'unparsed_paths':
        return t('quality.places.category.unparsed');
      case 'orphans':
        return t('quality.places.category.orphans');
      case 'type_mismatches':
        return t('quality.places.category.typeMismatch');
      case 'duplicates':
        return t('quality.places.category.duplicates');
      default:
        return cat;
    }
  };

  const getStepActionBadge = (action: string) => {
    if (action === 'reuse') {
      return (
        <Badge variant="neutral" size="sm">
          {t('quality.places.stepAction.reuse')}
        </Badge>
      );
    }
    if (action === 'create') {
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

  return (
    <Card
      data-testid={`place-hierarchy-card-${issue.place_id}`}
      className="p-4 sm:p-5 space-y-4"
    >
      {/* Card Header */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={tierVariant} size="sm">
            {t(`quality.places.confidence.${issue.confidence_tier}`, {
              score: scorePct,
              pct: scorePct,
            })}
          </Badge>
          <Badge variant="neutral" size="sm">
            {getCategoryLabel(issue.category)}
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
                  {t(`places.types.${issue.current_place_type || 'other'}`)}
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
          {issue.proposed_chain && issue.proposed_chain.length > 0 ? (
            <div className="space-y-1.5">
              {issue.proposed_chain.map((step, idx) => (
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
                    ({t(`places.types.${step.place_type || 'other'}`)})
                  </span>
                  {getStepActionBadge(step.action)}
                </div>
              ))}
            </div>
          ) : (
            <div className="text-xs text-secondary">
              {issue.proposed_clean_name} (
              {t(`places.types.${issue.proposed_place_type || 'other'}`)})
            </div>
          )}

          {/* Candidate Parents Selector for Medium/Ambiguous Issues */}
          {issue.candidate_parents && issue.candidate_parents.length > 1 && (
            <div className="pt-2 space-y-1.5 border-t border-border mt-2">
              <div className="text-xs font-medium text-secondary">
                {t('quality.places.selectCandidateParentLabel')}:
              </div>
              <div className="flex flex-wrap gap-1.5">
                {issue.candidate_parents.map((cand) => {
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
        <p className="text-xs text-secondary">{issue.rationale}</p>

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
