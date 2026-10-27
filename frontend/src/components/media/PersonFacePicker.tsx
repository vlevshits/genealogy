import React, { useMemo, useState } from 'react';
import { Check, Search, UserCheck, X } from 'lucide-react';
import { Avatar, Badge, Button, Checkbox, TextInput } from '../ui';
import { cn, matchesCrossScript } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface FaceTagCandidatePerson {
  id: string;
  gid: string;
  displayName: string;
  birthYear?: number | null;
  deathYear?: number | null;
  isLiving: boolean;
  avatarUrl?: string | null;
}

export const DEFAULT_FACE_CANDIDATES: FaceTagCandidatePerson[] = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    gid: 'I0001',
    displayName: 'Левшиц Михаил Александрович',
    birthYear: 1912,
    deathYear: 1989,
    isLiving: false,
    avatarUrl: '/api/v1/media/00000000-0000-0000-0000-000000000101/file',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    gid: 'I0002',
    displayName: 'Петров Александр Иванович',
    birthYear: 1912,
    deathYear: 1989,
    isLiving: false,
    avatarUrl: null,
  },
  {
    id: 'c7a8b9d0-1234-5678-90ab-cdef12345678',
    gid: 'I0042',
    displayName: 'Левшиц Елена Александровна',
    birthYear: 1994,
    deathYear: null,
    isLiving: true,
    avatarUrl: '/api/v1/media/00000000-0000-0000-0000-000000000102/file',
  },
];

export interface PersonFacePickerProps {
  candidates?: FaceTagCandidatePerson[];
  initialPersonId?: string;
  initialIsPrimaryAvatar?: boolean;
  submitting?: boolean;
  onConfirm: (personId: string, isPrimaryAvatar: boolean) => void;
  onCancel: () => void;
}

export const PersonFacePicker: React.FC<PersonFacePickerProps> = ({
  candidates = DEFAULT_FACE_CANDIDATES,
  initialPersonId = '',
  initialIsPrimaryAvatar = false,
  submitting = false,
  onConfirm,
  onCancel,
}) => {
  const { t } = useI18n();
  const [query, setQuery] = useState<string>('');
  const [selectedId, setSelectedId] = useState<string>(initialPersonId);
  const [isPrimaryAvatar, setIsPrimaryAvatar] = useState<boolean>(
    initialIsPrimaryAvatar
  );

  const filteredCandidates = useMemo(() => {
    const q = query.trim();
    if (!q) return candidates.slice(0, 12);
    return candidates
      .filter((c) => matchesCrossScript(`${c.displayName} ${c.gid}`, q))
      .slice(0, 12);
  }, [candidates, query]);

  const formatVitals = (person: FaceTagCandidatePerson): string => {
    const parts: string[] = [];
    if (person.birthYear) {
      parts.push(`${t('domain.bornPrefix')} ${person.birthYear}`);
    }
    if (!person.isLiving && person.deathYear) {
      parts.push(`† ${person.deathYear}`);
    }
    return parts.join(' · ');
  };

  return (
    <div
      role="dialog"
      aria-label={t('mediaArchive.facePickerTitle')}
      className="w-72 sm:w-80 rounded-2xl border border-border bg-surface p-3.5 shadow-2xl space-y-3 text-left"
    >
      <div className="flex items-center justify-between gap-2">
        <span className="inline-flex items-center gap-1.5 text-xs font-semibold text-primary">
          <UserCheck className="h-3.5 w-3.5 text-brand" />
          <span>{t('mediaArchive.facePickerTitle')}</span>
        </span>
        <Button
          variant="ghost"
          size="sm"
          onClick={onCancel}
          aria-label={t('common.cancel')}
          className="h-6 w-6 min-h-0 p-0 text-secondary"
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>

      <TextInput
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        leftIcon={<Search className="h-3.5 w-3.5 text-tertiary" />}
        placeholder={t('mediaArchive.facePickerSearchPlaceholder')}
        aria-label={t('mediaArchive.facePickerSearchPlaceholder')}
      />

      <div
        className="max-h-44 overflow-y-auto space-y-1 pr-0.5"
        role="listbox"
        aria-label={t('mediaArchive.facePickerTitle')}
      >
        {filteredCandidates.length === 0 ? (
          <p className="py-4 text-center text-xs text-secondary">
            {t('mediaArchive.facePickerEmpty')}
          </p>
        ) : (
          filteredCandidates.map((person) => {
            const isSelected = selectedId === person.id;
            const vitals = formatVitals(person);
            return (
              <div
                key={person.id}
                role="option"
                aria-selected={isSelected}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedId(person.id)}
                  className={cn(
                    'w-full justify-between gap-2 rounded-xl px-2.5 py-1.5 text-left font-normal',
                    isSelected
                      ? 'bg-brand/12 text-primary ring-1 ring-brand/40'
                      : 'text-primary hover:bg-surface-muted'
                  )}
                >
                  <span className="flex items-center gap-2.5 min-w-0">
                    <Avatar
                      name={person.displayName}
                      src={person.avatarUrl ?? undefined}
                      size="xs"
                    />
                    <span className="min-w-0">
                      <span className="block text-xs font-medium truncate">
                        {person.displayName}
                      </span>
                      {vitals && (
                        <span className="block text-[11px] text-secondary truncate">
                          {vitals}
                        </span>
                      )}
                    </span>
                  </span>
                  <Badge variant="neutral">{person.gid}</Badge>
                </Button>
              </div>
            );
          })
        )}
      </div>

      <div className="pt-1 border-t border-border space-y-2.5">
        <Checkbox
          checked={isPrimaryAvatar}
          onCheckedChange={setIsPrimaryAvatar}
          label={t('mediaArchive.setPrimaryAvatarCheckbox')}
        />

        <div className="flex items-center justify-end gap-2">
          <Button variant="secondary" size="sm" onClick={onCancel}>
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            size="sm"
            disabled={!selectedId || submitting}
            loading={submitting}
            onClick={() => {
              if (selectedId) {
                onConfirm(selectedId, isPrimaryAvatar);
              }
            }}
            leftIcon={<Check className="h-3.5 w-3.5" />}
          >
            {t('mediaArchive.saveFaceTagBtn')}
          </Button>
        </div>
      </div>
    </div>
  );
};
