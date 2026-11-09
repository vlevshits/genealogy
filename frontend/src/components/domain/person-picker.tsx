import React from 'react';
import { Combobox, ComboboxOption } from '../ui/combobox';
import { Individual } from '../../types/person';
import { PersonListItem } from './person-list-item';
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
  invalid?: boolean;
  className?: string;
  ariaLabel?: string;
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
}

export const PersonPicker: React.FC<PersonPickerProps> = ({
  people = [],
  value,
  onChange,
  excludeIds = [],
  onCreatePerson,
  loadPeople,
  placeholder,
  disabled = false,
  invalid = false,
  className,
  ariaLabel,
  hasMore = false,
  loadingMore = false,
  onLoadMore,
}) => {
  const { t } = useI18n();

  const toOption = (p: Individual): ComboboxOption<Individual> => {
    const rawLabel = (p as { display_name?: string }).display_name;
    const constructedName = [p.surname, p.given_name, p.patronymic, p.birth_surname]
      .filter(Boolean)
      .join(' ');
    const name = rawLabel || constructedName || p.id;
    const gidLabel = p.gid ? `(${p.gid})` : '';
    const label = gidLabel ? `${name} ${gidLabel}`.trim() : name;
    return {
      value: p.id,
      label,
      description: p.birth_place ?? undefined,
      data: p,
    };
  };

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
      invalid={invalid}
      className={className}
      ariaLabel={ariaLabel}
      hasMore={hasMore}
      loadingMore={loadingMore}
      onLoadMore={onLoadMore}
      renderOption={(opt, isSelected) =>
        opt.data ? <PersonListItem person={opt.data} selected={isSelected} /> : opt.label
      }
    />
  );
};
