import React, { useMemo } from 'react';
import { AlertCircle, UserCheck } from 'lucide-react';
import { Card, Checkbox } from '../ui';
import { PersonPicker } from '../domain/person-picker';
import { Individual } from '../../types/person';
import { useI18n } from '../../i18n';

export interface SpeakerOption {
  id: string;
  gid: string;
  display_name: string;
}

export interface SpeakerSelectorCardProps {
  speakers?: SpeakerOption[];
  people?: Individual[];
  selectedSpeakerId: string | null;
  onSelectSpeaker: (id: string | null, person?: Individual) => void;
  isNewSpeaker: boolean;
  onNewSpeakerChange: (isNew: boolean) => void;
  loadPeople?: (query: string, signal: AbortSignal) => Promise<Individual[]>;
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
  errorMessage?: string | null;
}

export const SpeakerSelectorCard: React.FC<SpeakerSelectorCardProps> = ({
  speakers,
  people,
  selectedSpeakerId,
  onSelectSpeaker,
  isNewSpeaker,
  onNewSpeakerChange,
  loadPeople,
  hasMore,
  loadingMore,
  onLoadMore,
  errorMessage,
}) => {
  const { t } = useI18n();

  const combinedPeople: Individual[] = useMemo(() => {
    if (people && people.length > 0) return people;
    return (speakers || []).map((sp) => ({
      id: sp.id,
      gid: sp.gid,
      display_name: sp.display_name,
      given_name: sp.display_name.split(' ')[0] || sp.display_name,
      sex: 'unknown',
      is_living: true,
      is_private: false,
      version: 1,
    }));
  }, [people, speakers]);

  const isInvalid = Boolean(errorMessage) && !isNewSpeaker;

  return (
    <Card
      data-testid="speaker-selector-card"
      className="p-4 sm:p-5 space-y-3"
    >
      <div className="flex items-start gap-3">
        <div className="h-9 w-9 rounded-lg bg-brand/10 flex items-center justify-center text-brand shrink-0">
          <UserCheck className="h-4.5 w-4.5" />
        </div>
        <div className="space-y-0.5 min-w-0">
          <h2 className="text-sm font-semibold text-primary">
            {t('stories.speaker.title')}
          </h2>
          <p className="text-xs text-secondary">
            {t('stories.speaker.subtitle')}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 items-start pt-1">
        <div className="min-w-0 space-y-1.5">
          <PersonPicker
            value={isNewSpeaker ? null : selectedSpeakerId}
            people={combinedPeople}
            loadPeople={loadPeople}
            hasMore={hasMore}
            loadingMore={loadingMore}
            onLoadMore={onLoadMore}
            onChange={(personId, person) => onSelectSpeaker(personId, person)}
            disabled={isNewSpeaker}
            invalid={isInvalid}
            placeholder={t('stories.speaker.searchPlaceholder')}
            ariaLabel={t('stories.speaker.selectAria')}
          />
          {isInvalid && (
            <p
              role="alert"
              data-testid="speaker-required-error"
              className="text-xs text-danger flex items-center gap-1.5 pt-0.5"
            >
              <AlertCircle className="h-3.5 w-3.5 shrink-0" />
              <span>{errorMessage}</span>
            </p>
          )}
        </div>

        <div className="pt-2.5">
          <Checkbox
            checked={isNewSpeaker}
            onCheckedChange={(checked) => onNewSpeakerChange(checked)}
            label={t('stories.speaker.newRelativeToggle')}
          />
        </div>
      </div>
    </Card>
  );
};
