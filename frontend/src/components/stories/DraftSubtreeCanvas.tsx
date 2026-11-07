import React from 'react';
import { GitBranch, Heart, User } from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import {
  DraftParentChildDto,
  DraftPersonNodeDto,
  DraftUnionDto,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface DraftSubtreeCanvasProps {
  people: DraftPersonNodeDto[];
  unions: DraftUnionDto[];
  parentChildLinks: DraftParentChildDto[];
  selectedTempId: string | null;
  onSelectPerson: (person: DraftPersonNodeDto) => void;
}

export const DraftSubtreeCanvas: React.FC<DraftSubtreeCanvasProps> = ({
  people = [],
  unions = [],
  parentChildLinks = [],
  selectedTempId,
  onSelectPerson,
}) => {
  const { t } = useI18n();

  // Group nodes by generation relative to speaker:
  // Parents (generation -1), Speaker & Partners/Peers (generation 0), Children (generation +1)
  const childIdsOfSpeaker = new Set(
    parentChildLinks
      .filter((l) => {
        const parent = people.find((p) => p.temp_id === l.parent_temp_id);
        return (
          parent?.role_to_speaker === 'speaker' ||
          parent?.role_to_speaker === 'spouse'
        );
      })
      .map((l) => l.child_temp_id)
  );

  const parentIdsOfSpeaker = new Set(
    parentChildLinks
      .filter((l) => {
        const child = people.find((p) => p.temp_id === l.child_temp_id);
        return child?.role_to_speaker === 'speaker';
      })
      .map((l) => l.parent_temp_id)
  );

  const ancestors = people.filter(
    (p) =>
      parentIdsOfSpeaker.has(p.temp_id) ||
      p.role_to_speaker === 'mother' ||
      p.role_to_speaker === 'father' ||
      p.role_to_speaker === 'parent'
  );

  const descendants = people.filter(
    (p) =>
      !ancestors.includes(p) &&
      (childIdsOfSpeaker.has(p.temp_id) ||
        p.role_to_speaker === 'child' ||
        p.role_to_speaker === 'daughter' ||
        p.role_to_speaker === 'son')
  );

  const peers = people.filter(
    (p) => !ancestors.includes(p) && !descendants.includes(p)
  );

  const renderNodeCard = (person: DraftPersonNodeDto) => {
    const score =
      person.confidence_score <= 1
        ? Math.round(person.confidence_score * 100)
        : Math.round(person.confidence_score);
    const isHigh = score >= 85;
    const isMedium = score >= 60 && score < 85;
    const isSelected = selectedTempId === person.temp_id;

    const borderClass = isSelected
      ? 'border-brand ring-2 ring-brand/25'
      : isHigh
        ? 'border-success/60'
        : isMedium
          ? 'border-warning/60'
          : 'border-border';

    return (
      <div
        key={person.temp_id}
        data-testid={`draft-canvas-node-${person.temp_id}`}
        className={`rounded-xl bg-surface p-3 border-2 shadow-xs transition-all min-w-[180px] max-w-[230px] ${borderClass}`}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0">
            <div className="h-7 w-7 rounded-full bg-brand/10 flex items-center justify-center text-brand shrink-0">
              <User className="h-3.5 w-3.5" />
            </div>
            <div className="min-w-0">
              <div className="text-xs font-semibold text-primary truncate">
                {person.given_name} {person.surname}
              </div>
              {person.patronymic && (
                <div className="text-[11px] text-secondary truncate">
                  {person.patronymic}
                </div>
              )}
            </div>
          </div>
          <Badge
            variant={isHigh ? 'success' : isMedium ? 'warning' : 'brand'}
            size="sm"
          >
            {score}%
          </Badge>
        </div>

        <div className="mt-2 space-y-1 text-[11px] text-secondary">
          <div>
            {person.birth_date || t('stories.canvas.unknownDate')}
            {person.birth_place ? ` · ${person.birth_place}` : ''}
          </div>
          {person.ambiguity_note && (
            <Badge variant="brand" size="sm">
              {person.ambiguity_note}
            </Badge>
          )}
        </div>

        <div className="mt-2 pt-2 border-t border-border flex items-center justify-between gap-2">
          <span className="text-[11px] font-medium text-tertiary">
            {person.role_to_speaker
              ? t(`stories.roles.${person.role_to_speaker}`)
              : t('stories.roles.relative')}
          </span>
          <Button
            size="sm"
            variant="ghost"
            data-testid={`draft-canvas-edit-btn-${person.temp_id}`}
            onClick={() => onSelectPerson(person)}
          >
            {t('common.edit')}
          </Button>
        </div>
      </div>
    );
  };

  return (
    <Card
      data-testid="draft-subtree-canvas"
      className="p-4 sm:p-5 space-y-4 flex flex-col"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <GitBranch className="h-4 w-4 text-brand" />
          <h3 className="text-sm font-semibold text-primary">
            {t('stories.canvas.title')}
          </h3>
        </div>
        <div className="flex flex-wrap items-center gap-1.5 text-xs">
          <Badge variant="success" size="sm">
            {t('stories.canvas.legendHigh')}
          </Badge>
          <Badge variant="warning" size="sm">
            {t('stories.canvas.legendMedium')}
          </Badge>
          <Badge variant="brand" size="sm">
            {t('stories.canvas.legendAmbiguous')}
          </Badge>
        </div>
      </div>

      {/* Interactive Staged Tree Diagram */}
      <div className="rounded-xl bg-canvas border border-border p-4 sm:p-6 overflow-x-auto min-h-[300px] flex flex-col items-center justify-center space-y-4">
        {/* Generation -1: Parents / Ancestors */}
        {ancestors.length > 0 && (
          <>
            <div className="flex flex-wrap items-center justify-center gap-4">
              {ancestors.map(renderNodeCard)}
            </div>
            <div
              aria-hidden="true"
              className="h-6 w-0.5 bg-border-card"
            />
          </>
        )}

        {/* Generation 0: Speaker & Spouse / Union */}
        <div className="flex flex-wrap items-center justify-center gap-3">
          {peers.map((peer, idx) => (
            <React.Fragment key={peer.temp_id}>
              {idx > 0 && unions.length > 0 && (
                <div
                  aria-hidden="true"
                  className="flex items-center justify-center h-7 w-7 rounded-full bg-surface border border-border text-brand"
                >
                  <Heart className="h-3.5 w-3.5" />
                </div>
              )}
              {renderNodeCard(peer)}
            </React.Fragment>
          ))}
        </div>

        {/* Generation +1: Children / Descendants */}
        {descendants.length > 0 && (
          <>
            <div
              aria-hidden="true"
              className="h-6 w-0.5 bg-border-card"
            />
            <div className="flex flex-wrap items-center justify-center gap-4">
              {descendants.map(renderNodeCard)}
            </div>
          </>
        )}
      </div>
    </Card>
  );
};
