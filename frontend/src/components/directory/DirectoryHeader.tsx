import React from 'react';
import { BookOpen, Plus, Search } from 'lucide-react';
import { Badge, Button, TextInput } from '../ui';
import { useI18n } from '../../i18n';

export interface DirectoryHeaderProps {
  total: number;
  totalUnfiltered: number;
  searchQuery: string;
  onSearchQueryChange: (value: string) => void;
  canEdit: boolean;
  onOpenSurnames: () => void;
  onAddPerson: () => void;
}

export const DirectoryHeader: React.FC<DirectoryHeaderProps> = ({
  total,
  totalUnfiltered,
  searchQuery,
  onSearchQueryChange,
  canEdit,
  onOpenSurnames,
  onAddPerson,
}) => {
  const { t, locale } = useI18n();
  const formatNum = (n: number) =>
    n.toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US');
  const isFiltered = total !== totalUnfiltered && totalUnfiltered > 0;

  return (
    <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
      <div className="space-y-1">
        <div className="flex flex-wrap items-center gap-2.5">
          <h1 className="font-sans text-2xl sm:text-3xl font-semibold tracking-tight text-primary">
            {t('directory.title')}
          </h1>
          <Badge variant="neutral">
            {isFiltered
              ? t('directory.countFiltered', {
                  shown: formatNum(total),
                  total: formatNum(totalUnfiltered),
                })
              : formatNum(total)}
          </Badge>
        </div>
        <p className="text-sm text-secondary">{t('directory.subtitle')}</p>
      </div>

      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5">
        <div className="w-full sm:w-72">
          <TextInput
            value={searchQuery}
            onChange={(e) => onSearchQueryChange(e.target.value)}
            leftIcon={<Search className="h-4 w-4 text-tertiary" />}
            placeholder={t('directory.searchPlaceholder')}
            aria-label={t('directory.searchPlaceholder')}
          />
        </div>

        <Button
          variant="secondary"
          size="md"
          onClick={onOpenSurnames}
          leftIcon={<BookOpen className="h-4 w-4" />}
        >
          {t('directory.surnamesIndexBtn')}
        </Button>

        {canEdit && (
          <Button
            variant="primary"
            size="md"
            onClick={onAddPerson}
            leftIcon={<Plus className="h-4 w-4" />}
          >
            {t('directory.addPersonBtn')}
          </Button>
        )}
      </div>
    </div>
  );
};
