import React, { useEffect, useState } from 'react';
import { AlertTriangle, ExternalLink, Link2 } from 'lucide-react';
import { Alert, Badge, Button } from '../ui';
import {
  DuplicateQuickMatchDto,
  peopleApi,
  SexCode,
  TreePersonNodeDto,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface QuickAddDuplicateHintProps {
  givenName: string;
  surname: string;
  birthDate?: string;
  sex?: SexCode;
  anchorPersonId?: string | null;
  existingPeople?: TreePersonNodeDto[];
  onUseExisting?: (personId: string) => void;
  onOpenProfile?: (personId: string) => void;
}

export const QuickAddDuplicateHint: React.FC<QuickAddDuplicateHintProps> = ({
  givenName,
  surname,
  birthDate = '',
  sex = 'U',
  anchorPersonId,
  existingPeople = [],
  onUseExisting,
  onOpenProfile,
}) => {
  const { t } = useI18n();
  const [remoteMatches, setRemoteMatches] = useState<DuplicateQuickMatchDto[]>(
    []
  );

  // Extract 4-digit birth year if present
  const birthYearMatch = birthDate.match(/\b(1[5-9]\d{2}|20\d{2})\b/);
  const birthYear = birthYearMatch ? Number(birthYearMatch[1]) : null;

  useEffect(() => {
    const trimmedGiven = givenName.trim();
    if (trimmedGiven.length < 2) {
      setRemoteMatches([]);
      return;
    }

    let cancelled = false;
    const timer = setTimeout(() => {
      const fullQuery = [trimmedGiven, surname.trim()]
        .filter(Boolean)
        .join(' ');
      peopleApi
        .findDuplicates(fullQuery, birthYear, sex)
        .then((items) => {
          if (!cancelled) {
            setRemoteMatches(
              items.filter((m) => (m.person?.id ?? m.id) !== anchorPersonId)
            );
          }
        })
        .catch(() => {
          if (!cancelled) setRemoteMatches([]);
        });
    }, 220);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [givenName, surname, birthYear, sex, anchorPersonId]);

  // Also check local existingPeople immediately
  const localMatch =
    givenName.trim().length >= 2
      ? existingPeople.find(
          (p) =>
            p.id !== anchorPersonId &&
            p.primary_name.given_name.toLowerCase() ===
              givenName.trim().toLowerCase() &&
            (!surname.trim() ||
              p.primary_name.surname.toLowerCase() ===
                surname.trim().toLowerCase())
        )
      : undefined;

  const topRemote = remoteMatches[0];
  const candidate = topRemote
    ? {
        id: topRemote.person?.id ?? topRemote.id,
        gid: topRemote.person?.gid ?? topRemote.gid,
        displayName:
          topRemote.person?.display_name ||
          topRemote.display_name ||
          [
            topRemote.person?.given_name ?? topRemote.given_name,
            topRemote.person?.patronymic ?? topRemote.patronymic,
            topRemote.person?.surname ?? topRemote.surname,
          ]
            .filter(Boolean)
            .join(' '),
        birthDate: topRemote.person?.birth_date ?? topRemote.birth_date ?? null,
        deathDate: topRemote.person?.death_date ?? topRemote.death_date ?? null,
        isLiving: topRemote.person?.is_living ?? topRemote.is_living ?? false,
        score:
          typeof topRemote.score === 'number'
            ? topRemote.score <= 1
              ? Math.round(topRemote.score * 100)
              : Math.round(topRemote.score)
            : 88,
      }
    : localMatch
    ? {
        id: localMatch.id,
        gid: localMatch.gid,
        displayName: [
          localMatch.primary_name.given_name,
          localMatch.primary_name.patronymic,
          localMatch.primary_name.surname,
        ]
          .filter(Boolean)
          .join(' '),
        birthDate: localMatch.birth?.date ?? null,
        deathDate: localMatch.death?.date ?? null,
        isLiving: localMatch.is_living,
        score: 90,
      }
    : null;

  if (!candidate) return null;

  const lifespanLabel = [
    candidate.birthDate ? candidate.birthDate.slice(0, 4) : null,
    !candidate.isLiving && candidate.deathDate
      ? `†${candidate.deathDate.slice(0, 4)}`
      : null,
  ]
    .filter(Boolean)
    .join(' – ');

  return (
    <Alert
      variant="warning"
      data-testid="quick-add-duplicate-hint"
      icon={
        <AlertTriangle className="h-4 w-4 text-warning shrink-0 mt-0.5" />
      }
      title={t('person.quickAdd.duplicateGuardTitle')}
      action={
        <div className="flex flex-wrap items-center gap-2">
          {onOpenProfile && (
            <Button
              size="sm"
              variant="ghost"
              data-testid="duplicate-hint-open-profile-btn"
              leftIcon={<ExternalLink className="h-3.5 w-3.5" />}
              onClick={() => onOpenProfile(candidate.id)}
            >
              {t('quality.duplicates.viewProfileBtn')}
            </Button>
          )}
          {onUseExisting && (
            <Button
              size="sm"
              variant="secondary"
              data-testid="duplicate-hint-use-existing-btn"
              leftIcon={<Link2 className="h-3.5 w-3.5" />}
              onClick={() => onUseExisting(candidate.id)}
            >
              {t('person.quickAdd.useExistingBtn')}
            </Button>
          )}
        </div>
      }
    >
      <div className="flex flex-wrap items-center gap-2">
        <span>
          {t('person.quickAdd.duplicateGuardDesc', {
            name: candidate.displayName,
            gid: candidate.gid,
          })}
        </span>
        {lifespanLabel && (
          <Badge variant="neutral" size="sm">
            {lifespanLabel}
          </Badge>
        )}
        <Badge variant="warning" size="sm">
          {candidate.score}%
        </Badge>
      </div>
    </Alert>
  );
};
