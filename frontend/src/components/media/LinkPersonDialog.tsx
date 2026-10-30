import React, { useMemo, useState } from 'react';
import { Check, Search, UserPlus } from 'lucide-react';
import { MediaAssetSummaryDto } from '../../api/client';
import { Avatar, Badge, Button, Dialog, TextInput } from '../ui';
import { cn, matchesCrossScript } from '../../lib/utils';
import { useI18n } from '../../i18n';
import { FaceTagCandidatePerson } from './PersonFacePicker';

export interface LinkPersonDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: MediaAssetSummaryDto;
  candidates: FaceTagCandidatePerson[];
  onLinkPerson: (personId: string) => Promise<void> | void;
}

export const LinkPersonDialog: React.FC<LinkPersonDialogProps> = ({
  open,
  onOpenChange,
  asset,
  candidates,
  onLinkPerson,
}) => {
  const { t } = useI18n();
  const [query, setQuery] = useState<string>('');
  const [selectedPersonId, setSelectedPersonId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState<boolean>(false);

  // Compute set of individuals already linked or tagged in this asset
  const alreadyAssociatedIds = useMemo(() => {
    const ids = new Set<string>(asset.linked_person_ids || []);
    for (const tag of asset.face_tags || []) {
      ids.add(tag.person_id);
    }
    for (const link of asset.links || []) {
      if (link.entity_type === 'individual') {
        ids.add(link.entity_id);
      }
    }
    return ids;
  }, [asset]);

  // Filter candidates to only untagged, unlinked individuals
  const eligibleCandidates = useMemo(() => {
    return candidates.filter((cand) => !alreadyAssociatedIds.has(cand.id));
  }, [candidates, alreadyAssociatedIds]);

  const filteredCandidates = useMemo(() => {
    const q = query.trim();
    if (!q) return eligibleCandidates;
    return eligibleCandidates.filter((cand) =>
      matchesCrossScript(`${cand.displayName} ${cand.gid}`, q)
    );
  }, [eligibleCandidates, query]);

  const formatVitals = (cand: FaceTagCandidatePerson): string => {
    const parts: string[] = [];
    if (cand.birthYear) {
      parts.push(`${t('domain.bornPrefix')} ${cand.birthYear}`);
    }
    if (!cand.isLiving && cand.deathYear) {
      parts.push(`† ${cand.deathYear}`);
    }
    return parts.join(' · ');
  };

  const handleConfirm = async () => {
    if (!selectedPersonId || submitting) return;
    setSubmitting(true);
    try {
      await onLinkPerson(selectedPersonId);
      setSelectedPersonId(null);
      setQuery('');
      onOpenChange(false);
    } finally {
      setSubmitting(false);
    }
  };

  const handleListKeyDown = (e: React.KeyboardEvent) => {
    if (filteredCandidates.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const currentIndex = filteredCandidates.findIndex(
        (c) => c.id === selectedPersonId
      );
      const nextIndex =
        currentIndex < filteredCandidates.length - 1 ? currentIndex + 1 : 0;
      setSelectedPersonId(filteredCandidates[nextIndex].id);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const currentIndex = filteredCandidates.findIndex(
        (c) => c.id === selectedPersonId
      );
      const prevIndex =
        currentIndex > 0 ? currentIndex - 1 : filteredCandidates.length - 1;
      setSelectedPersonId(filteredCandidates[prevIndex].id);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!submitting) {
          if (!nextOpen) {
            setSelectedPersonId(null);
            setQuery('');
          }
          onOpenChange(nextOpen);
        }
      }}
      size="md"
      title={t('mediaArchive.linkPersonTitle')}
      description={t('mediaArchive.linkPersonSubtitle')}
      footer={
        <div className="flex items-center justify-end gap-2 w-full">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onOpenChange(false)}
            disabled={submitting}
            className="touch-target-expand"
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            size="sm"
            disabled={!selectedPersonId || submitting}
            loading={submitting}
            onClick={() => void handleConfirm()}
            leftIcon={<Check className="h-3.5 w-3.5" />}
            className="touch-target-expand"
          >
            {t('mediaArchive.linkPersonSubmit')}
          </Button>
        </div>
      }
    >
      <div className="space-y-3">
        <TextInput
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
              handleListKeyDown(e);
            }
          }}
          placeholder={t('mediaArchive.linkPersonSearchPlaceholder')}
          leftIcon={<Search className="h-3.5 w-3.5 text-tertiary" />}
          aria-label={t('mediaArchive.linkPersonSearchPlaceholder')}
        />

        <div
          role="listbox"
          tabIndex={0}
          onKeyDown={handleListKeyDown}
          aria-label={t('mediaArchive.linkPersonTitle')}
          className="max-h-64 overflow-y-auto space-y-1.5 pr-0.5 border border-border/60 rounded-xl p-1.5 bg-surface-muted/30"
        >
          {filteredCandidates.length === 0 ? (
            <div className="py-8 text-center text-xs text-secondary flex flex-col items-center gap-1.5">
              <UserPlus className="h-6 w-6 text-tertiary" />
              <span>{t('mediaArchive.linkPersonEmpty')}</span>
            </div>
          ) : (
            filteredCandidates.map((cand) => {
              const isSelected = selectedPersonId === cand.id;
              const vitals = formatVitals(cand);

              return (
                <Button
                  key={cand.id}
                  role="option"
                  aria-selected={isSelected}
                  tabIndex={isSelected ? 0 : -1}
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedPersonId(cand.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                      handleListKeyDown(e);
                    }
                  }}
                  className={cn(
                    'w-full justify-between gap-2.5 rounded-xl px-3 py-2 text-left font-normal transition-colors',
                    isSelected
                      ? 'bg-brand/12 text-primary ring-1 ring-brand/50'
                      : 'text-primary hover:bg-surface-muted'
                  )}
                >
                  <span className="flex items-center gap-3 min-w-0">
                    <Avatar
                      name={cand.displayName}
                      src={cand.avatarUrl ?? undefined}
                      size="sm"
                    />
                    <span className="min-w-0">
                      <span className="block text-xs font-semibold truncate">
                        {cand.displayName}
                      </span>
                      {vitals && (
                        <span className="block text-[11px] text-secondary truncate">
                          {vitals}
                        </span>
                      )}
                    </span>
                  </span>

                  <div className="flex items-center gap-2 shrink-0">
                    <Badge variant="neutral">{cand.gid}</Badge>
                    {isSelected && (
                      <Check className="h-4 w-4 text-brand" />
                    )}
                  </div>
                </Button>
              );
            })
          )}
        </div>
      </div>
    </Dialog>
  );
};
