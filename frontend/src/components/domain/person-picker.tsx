import React from 'react';
import { Combobox, ComboboxOption } from '../ui/combobox';
import { Individual } from '../../types/person';
import { PersonListItem } from './person-list-item';
import { SAMPLE_PEOPLE } from '../../lib/mock-data';
import { matchesCrossScript } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface PersonPickerProps {
  people?: Individual[];
  value?: string | null;
  onChange?: (personId: string | null, person?: Individual) => void;
  excludeIds?: string[];
  onCreatePerson?: (query: string) => void;
  loadPeople?: (query: string, signal: AbortSignal) => Promise<Individual[]>;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export const PersonPicker: React.FC<PersonPickerProps> = ({
  people = SAMPLE_PEOPLE,
  value,
  onChange,
  excludeIds = [],
  onCreatePerson,
  loadPeople,
  placeholder,
  disabled = false,
  className,
}) => {
  const { t } = useI18n();

  const toOption = (p: Individual): ComboboxOption<Individual> => ({
    value: p.id,
    label: [p.surname, p.given_name, p.patronymic, p.birth_surname, p.gid]
      .filter(Boolean)
      .join(' '),
    description: p.birth_place ?? undefined,
    data: p,
  });

  const filteredPool = people.filter((p) => !excludeIds.includes(p.id));
  const staticOptions = filteredPool.map(toOption);

  const asyncLoader = async (
    query: string,
    signal: AbortSignal
  ): Promise<ComboboxOption<Individual>[]> => {
    if (loadPeople) {
      const list = await loadPeople(query, signal);
      return list.filter((p) => !excludeIds.includes(p.id)).map(toOption);
    }
    return staticOptions.filter((opt) => matchesCrossScript(opt.label, query));
  };

  return (
    <Combobox<Individual>
      value={value}
      options={staticOptions}
      loadOptions={loadPeople ? asyncLoader : undefined}
      onChange={(val, opt) => onChange?.(val, opt?.data)}
      onCreate={onCreatePerson}
      placeholder={placeholder || t('nav.searchTrigger')}
      disabled={disabled}
      className={className}
      renderOption={(opt, isSelected) =>
        opt.data ? <PersonListItem person={opt.data} selected={isSelected} /> : opt.label
      }
    />
  );
};
