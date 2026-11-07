import React from 'react';
import {
  CheckCircle2,
  GitMerge,
  Trash2,
  UserPlus,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  Checkbox,
  SegmentedControl,
} from '../ui';
import {
  DraftPersonNodeDto,
  StoryReconciliationCandidateDto,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface ReconciliationResolutionState {
  action: 'link_existing' | 'create_new';
  matched_person_id?: string | null;
  enrich_fields: string[];
}

export interface ReconciliationMatrixCardProps {
  people: DraftPersonNodeDto[];
  candidates: StoryReconciliationCandidateDto[];
  resolutions: Record<string, ReconciliationResolutionState>;
  onResolutionChange: (
    tempId: string,
    next: ReconciliationResolutionState
  ) => void;
  committing?: boolean;
  onCommit: () => void;
  onDiscard: () => void;
}

export const ReconciliationMatrixCard: React.FC<
  ReconciliationMatrixCardProps
> = ({
  people,
  candidates,
  resolutions,
  onResolutionChange,
  committing = false,
  onCommit,
  onDiscard,
}) => {
  const { t } = useI18n();

  const resolutionValues = Object.values(resolutions);
  const linkedCount = resolutionValues.filter(
    (r) => r.action === 'link_existing'
  ).length;
  const createdCount = resolutionValues.filter(
    (r) => r.action === 'create_new'
  ).length;

  return (
    <Card
      data-testid="reconciliation-matrix-card"
      className="p-4 sm:p-5 space-y-5"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2">
            <GitMerge className="h-4.5 w-4.5 text-brand" />
            <h2 className="text-base font-semibold text-primary">
              {t('stories.reconcile.title')}
            </h2>
          </div>
          <p className="text-xs text-secondary">
            {t('stories.reconcile.subtitle')}
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Badge variant="brand" size="sm">
            {t('stories.reconcile.summaryLinked', { count: linkedCount })}
          </Badge>
          <Badge variant="success" size="sm">
            {t('stories.reconcile.summaryNew', { count: createdCount })}
          </Badge>
        </div>
      </div>

      {/* Rows per Draft Individual */}
      <div className="space-y-3">
        {people.map((person) => {
          const cand = candidates.find(
            (c) => c.draft_temp_id === person.temp_id
          );
          const hasExistingMatch = Boolean(
            cand?.matched_person_id && cand.similarity_score >= 60
          );
          const currentRes: ReconciliationResolutionState = resolutions[
            person.temp_id
          ] ?? {
            action: hasExistingMatch ? 'link_existing' : 'create_new',
            matched_person_id: cand?.matched_person_id ?? null,
            enrich_fields: cand?.enrich_fields ?? ['birth_place', 'notes'],
          };

          const toggleEnrichField = (field: string) => {
            const exists = currentRes.enrich_fields.includes(field);
            const nextFields = exists
              ? currentRes.enrich_fields.filter((f) => f !== field)
              : [...currentRes.enrich_fields, field];
            onResolutionChange(person.temp_id, {
              ...currentRes,
              enrich_fields: nextFields,
            });
          };

          return (
            <div
              key={person.temp_id}
              data-testid={`reconcile-row-${person.temp_id}`}
              className="rounded-xl border border-border bg-surface-muted p-4 grid grid-cols-1 lg:grid-cols-12 gap-4 items-center"
            >
              {/* Col 1: Draft Person */}
              <div className="lg:col-span-4 space-y-1 min-w-0">
                <div className="text-sm font-semibold text-primary">
                  {person.given_name}{' '}
                  {person.patronymic ? `${person.patronymic} ` : ''}
                  {person.surname}
                </div>
                <div className="text-xs text-secondary">
                  {person.birth_date || t('stories.canvas.unknownDate')}
                  {person.birth_place ? ` · ${person.birth_place}` : ''}
                </div>
              </div>

              {/* Col 2: Proposed Action Toggle */}
              <div className="lg:col-span-4 flex flex-col gap-2">
                <SegmentedControl<'link_existing' | 'create_new'>
                  size="sm"
                  ariaLabel={t('stories.reconcile.actionAria')}
                  value={currentRes.action}
                  onChange={(nextAction) =>
                    onResolutionChange(person.temp_id, {
                      ...currentRes,
                      action: nextAction,
                    })
                  }
                  options={[
                    {
                      value: 'link_existing',
                      label: t('stories.reconcile.actionLink'),
                      disabled: !hasExistingMatch,
                    },
                    {
                      value: 'create_new',
                      label: t('stories.reconcile.actionCreate'),
                      icon: <UserPlus className="h-3.5 w-3.5" />,
                    },
                  ]}
                />

                {currentRes.action === 'link_existing' && (
                  <div className="flex flex-wrap items-center gap-3 pt-1">
                    <Checkbox
                      checked={currentRes.enrich_fields.includes('birth_place')}
                      onCheckedChange={() => toggleEnrichField('birth_place')}
                      label={t('stories.reconcile.enrichBirthPlace')}
                    />
                    <Checkbox
                      checked={currentRes.enrich_fields.includes('notes')}
                      onCheckedChange={() => toggleEnrichField('notes')}
                      label={t('stories.reconcile.enrichNotes')}
                    />
                  </div>
                )}
              </div>

              {/* Col 3: Match Details & Relationship Context */}
              <div className="lg:col-span-4 space-y-1 text-xs">
                {hasExistingMatch && cand ? (
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Badge variant="success" size="sm">
                      {t('stories.reconcile.matchScoreBadge', {
                        score: cand.similarity_score,
                      })}
                    </Badge>
                    <span className="font-medium text-primary">
                      {cand.matched_person_name}
                    </span>
                    {cand.matched_person_gid && (
                      <span className="font-mono text-tertiary">
                        ({cand.matched_person_gid})
                      </span>
                    )}
                  </div>
                ) : (
                  <Badge variant="neutral" size="sm">
                    {t('stories.reconcile.netNewBadge')}
                  </Badge>
                )}

                {cand?.target_anchor_label && (
                  <div className="text-secondary">
                    {cand.target_anchor_label}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Footer Commit & Discard */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-border">
        <Button
          variant="ghost"
          size="sm"
          data-testid="discard-story-draft-btn"
          leftIcon={<Trash2 className="h-4 w-4" />}
          onClick={onDiscard}
        >
          {t('stories.reconcile.discardBtn')}
        </Button>

        <Button
          variant="primary"
          size="md"
          loading={committing}
          data-testid="commit-story-subtree-btn"
          leftIcon={<CheckCircle2 className="h-4 w-4" />}
          onClick={onCommit}
        >
          {t('stories.reconcile.commitBtn', {
            newCount: createdCount,
            linkedCount: linkedCount,
          })}
        </Button>
      </div>
    </Card>
  );
};
