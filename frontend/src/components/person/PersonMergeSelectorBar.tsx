import React, { useMemo, useState } from 'react';
import {
  ArrowLeftRight,
  Calendar,
  Camera,
  FileText,
  Search,
  Sparkles,
  Users,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Card,
  Select,
  SelectOption,
  TextInput,
} from '../ui';
import { PersonMergePreviewDto, TreePersonNodeDto } from '../../api/client';
import { useI18n } from '../../i18n';
import {
  formatTreePersonLabel,
  rankMergeCandidatesForTarget,
} from './person-merge-helpers';

export interface PersonMergeSelectorBarProps {
  people: TreePersonNodeDto[];
  targetId: string;
  sourceId: string;
  onSelectTarget: (newTargetId: string) => void;
  onSelectSource: (newSourceId: string) => void;
  onSwapDirection: () => void;
}

export const PersonMergeSelectorBar: React.FC<PersonMergeSelectorBarProps> = ({
  people,
  targetId,
  sourceId,
  onSelectTarget,
  onSelectSource,
  onSwapDirection,
}) => {
  const { t } = useI18n();
  const [searchQuery, setSearchQuery] = useState<string>('');

  const targetNode = useMemo(
    () => people.find((p) => p.id === targetId || p.gid === targetId),
    [people, targetId]
  );

  const rankedCandidates = useMemo(
    () => rankMergeCandidatesForTarget(targetNode, people),
    [targetNode, people]
  );

  const normalizedQuery = searchQuery.trim().toLowerCase();

  const filteredCandidates = useMemo(() => {
    if (!normalizedQuery) return rankedCandidates;
    return rankedCandidates.filter(({ person }) => {
      const label = formatTreePersonLabel(person).toLowerCase();
      const birthSurname = (person.birth_surname || '').toLowerCase();
      return (
        label.includes(normalizedQuery) ||
        person.id.toLowerCase().includes(normalizedQuery) ||
        person.gid.toLowerCase().includes(normalizedQuery) ||
        birthSurname.includes(normalizedQuery)
      );
    });
  }, [rankedCandidates, normalizedQuery]);

  const targetOptions = useMemo<SelectOption<string>[]>(() => {
    const base: SelectOption<string>[] = people.map((p) => ({
      value: p.id,
      label: formatTreePersonLabel(p),
      description: p.birth?.place || undefined,
    }));
    if (targetId && !base.some((o) => o.value === targetId)) {
      base.unshift({
        value: targetId,
        label: targetId,
        description: undefined,
      });
    }
    return base;
  }, [people, targetId]);

  const sourceOptions = useMemo<SelectOption<string>[]>(() => {
    const list: SelectOption<string>[] = filteredCandidates.map(
      ({ person, score }) => ({
        value: person.id,
        label: formatTreePersonLabel(person),
        description:
          score >= 15
            ? `${t('merge.suggestedMatchBadge')} · ${person.birth?.place || person.gid}`
            : person.birth?.place || undefined,
      })
    );

    if (sourceId && !list.some((o) => o.value === sourceId)) {
      const existingSource = people.find(
        (p) => p.id === sourceId || p.gid === sourceId
      );
      list.unshift({
        value: existingSource ? existingSource.id : sourceId,
        label: existingSource ? formatTreePersonLabel(existingSource) : sourceId,
        description: existingSource?.birth?.place || undefined,
      });
    }
    return list;
  }, [filteredCandidates, people, sourceId, t]);

  const topQuickPicks = useMemo(() => {
    if (normalizedQuery) {
      return filteredCandidates.slice(0, 6);
    }
    return rankedCandidates.filter((c) => c.score >= 15).slice(0, 4);
  }, [normalizedQuery, filteredCandidates, rankedCandidates]);

  return (
    <Card
      className="p-4 sm:p-5 space-y-4"
      data-testid="person-merge-selector-bar"
    >
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2">
            <Users className="h-4 w-4 text-brand shrink-0" />
            <h2 className="text-sm font-semibold text-primary">
              {t('merge.selectorTitle')}
            </h2>
          </div>
          <p className="text-xs text-secondary">{t('merge.selectorSubtitle')}</p>
        </div>

        <Button
          variant="secondary"
          size="sm"
          leftIcon={<ArrowLeftRight className="h-3.5 w-3.5" />}
          onClick={onSwapDirection}
          data-testid="merge-swap-direction-btn"
        >
          {t('merge.swapDirectionBtn')}
        </Button>
      </div>

      {/* Candidate Search Input */}
      <div className="space-y-2">
        <TextInput
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          leftIcon={<Search className="h-4 w-4" />}
          placeholder={t('merge.searchCandidatePlaceholder')}
          aria-label={t('merge.searchCandidatePlaceholder')}
          data-testid="merge-candidate-search-input"
        />

        {topQuickPicks.length > 0 && (
          <div
            className="flex flex-wrap items-center gap-1.5"
            data-testid="merge-candidate-quick-picks"
          >
            {topQuickPicks.map(({ person, score }) => {
              const isSelected =
                person.id === sourceId || person.gid === sourceId;
              return (
                <Button
                  key={person.id}
                  size="xs"
                  variant={isSelected ? 'primary' : 'secondary'}
                  onClick={() => onSelectSource(person.id)}
                  data-testid={`merge-quick-pick-${person.id}`}
                >
                  <span className="truncate max-w-[220px]">
                    {formatTreePersonLabel(person)}
                  </span>
                  {score >= 15 && !isSelected && (
                    <Badge
                      variant="brand"
                      size="sm"
                      icon={<Sparkles className="h-2.5 w-2.5" />}
                      className="ml-1"
                    >
                      {t('merge.suggestedMatchBadge')}
                    </Badge>
                  )}
                </Button>
              );
            })}
          </div>
        )}
      </div>

      {/* Primary (Target) & Duplicate (Source) Selectors */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5 items-end">
        <div className="space-y-1.5">
          <div className="text-xs font-semibold text-secondary">
            {t('merge.targetSelectLabel')}
          </div>
          <Select<string>
            id="merge-target-select"
            ariaLabel={t('merge.targetSelectLabel')}
            value={targetNode ? targetNode.id : targetId}
            options={targetOptions}
            onChange={(val) => onSelectTarget(val)}
          />
        </div>

        <div className="space-y-1.5">
          <div className="text-xs font-semibold text-secondary">
            {t('merge.sourceSelectLabel')}
          </div>
          <Select<string>
            id="merge-source-select"
            ariaLabel={t('merge.sourceSelectLabel')}
            value={sourceId}
            options={sourceOptions}
            onChange={(val) => onSelectSource(val)}
          />
        </div>
      </div>

      {targetId && sourceId && targetId === sourceId && (
        <Alert variant="warning">{t('merge.samePersonWarning')}</Alert>
      )}
    </Card>
  );
};

export const PersonMergeCombinedUnionCard: React.FC<{
  counts: PersonMergePreviewDto['combined_counts'];
}> = ({ counts }) => {
  const { t } = useI18n();
  const items = [
    {
      key: 'events',
      icon: <Calendar className="h-4 w-4 text-brand shrink-0" />,
      label: t('merge.combinedEvents', { count: counts.events }),
      value: counts.events,
    },
    {
      key: 'media',
      icon: <Camera className="h-4 w-4 text-brand shrink-0" />,
      label: t('merge.combinedMedia', { count: counts.media }),
      value: counts.media,
    },
    {
      key: 'families',
      icon: <Users className="h-4 w-4 text-brand shrink-0" />,
      label: t('merge.combinedFamilies', { count: counts.families }),
      value: counts.families,
    },
    {
      key: 'notes',
      icon: <FileText className="h-4 w-4 text-brand shrink-0" />,
      label: t('merge.combinedNotes', { count: counts.notes }),
      value: counts.notes,
    },
  ];

  return (
    <Card className="p-5 space-y-3">
      <div className="text-xs font-semibold text-secondary uppercase tracking-wide">
        {t('merge.combinedPreviewHeading')}
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {items.map((item) => (
          <div
            key={item.key}
            className="rounded-xl border border-border bg-surface-muted/40 p-3 flex items-center gap-2.5"
          >
            {item.icon}
            <div>
              <div className="text-xs text-secondary">{item.label}</div>
              <div className="font-mono text-sm font-bold text-primary">
                {item.value}
              </div>
            </div>
          </div>
        ))}
      </div>
      <p className="text-xs text-secondary">{t('merge.altNamePreservedHint')}</p>
    </Card>
  );
};
