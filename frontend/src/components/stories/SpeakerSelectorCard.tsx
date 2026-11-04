import React from 'react';
import { UserCheck } from 'lucide-react';
import { Card, Checkbox, Select } from '../ui';
import { useI18n } from '../../i18n';

export interface SpeakerOption {
  id: string;
  gid: string;
  display_name: string;
}

export interface SpeakerSelectorCardProps {
  speakers: SpeakerOption[];
  selectedSpeakerId: string;
  onSelectSpeaker: (id: string) => void;
  isNewSpeaker: boolean;
  onNewSpeakerChange: (isNew: boolean) => void;
}

export const SpeakerSelectorCard: React.FC<SpeakerSelectorCardProps> = ({
  speakers,
  selectedSpeakerId,
  onSelectSpeaker,
  isNewSpeaker,
  onNewSpeakerChange,
}) => {
  const { t } = useI18n();

  const selectOptions = speakers.map((sp) => ({
    value: sp.id,
    label: `${sp.display_name} (${sp.gid})`,
  }));

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

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 items-center pt-1">
        <div className="min-w-0">
          <Select
            value={selectedSpeakerId}
            onChange={onSelectSpeaker}
            options={selectOptions}
            disabled={isNewSpeaker}
            ariaLabel={t('stories.speaker.selectAria')}
          />
        </div>

        <div>
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
