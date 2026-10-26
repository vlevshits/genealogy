import React from 'react';
import { MapPin } from 'lucide-react';
import { Combobox, ComboboxOption } from '../ui/combobox';
import { Place } from '../../types/place';
import { SAMPLE_PLACES } from '../../lib/mock-data';
import { useI18n } from '../../i18n';

export interface PlacePickerProps {
  places?: Place[];
  value?: string | null;
  onChange?: (placeId: string | null, place?: Place) => void;
  onCreatePlace?: (name: string) => void;
  loadPlaces?: (query: string, signal: AbortSignal) => Promise<Place[]>;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export const PlacePicker: React.FC<PlacePickerProps> = ({
  places = SAMPLE_PLACES,
  value,
  onChange,
  onCreatePlace,
  loadPlaces,
  placeholder,
  disabled = false,
  className,
}) => {
  const { t } = useI18n();

  const toOption = (pl: Place): ComboboxOption<Place> => ({
    value: pl.id,
    label: pl.name,
    description: pl.hierarchy_names?.join(', '),
    data: pl,
  });

  const staticOptions = places.map(toOption);

  return (
    <Combobox<Place>
      value={value}
      options={staticOptions}
      loadOptions={
        loadPlaces
          ? async (q, sig) => {
              const list = await loadPlaces(q, sig);
              return list.map(toOption);
            }
          : undefined
      }
      onChange={(val, opt) => onChange?.(val, opt?.data)}
      onCreate={onCreatePlace}
      placeholder={placeholder || t('domain.eventForm.placeLabel')}
      disabled={disabled}
      className={className}
      renderOption={(opt) => (
        <div className="flex items-start gap-2.5">
          <MapPin className="mt-0.5 h-4 w-4 shrink-0 text-accent" />
          <div className="min-w-0">
            <div className="font-medium text-primary truncate">{opt.label}</div>
            {opt.description && (
              <div className="text-[11px] text-secondary truncate">{opt.description}</div>
            )}
          </div>
        </div>
      )}
    />
  );
};
