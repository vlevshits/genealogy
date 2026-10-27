import React, { useEffect, useMemo, useState } from 'react';
import { ArrowLeft, Calendar, Search, Users } from 'lucide-react';
import {
  fetchSurnamesIndex,
  SurnameLetterGroupDto,
  SurnamesIndexResponse,
} from '../api/client';
import { Badge, Button, Card, EmptyState, TextInput } from '../components/ui';
import { matchesCrossScript } from '../lib/utils';
import { useI18n } from '../i18n';

export const DEFAULT_SURNAMES_INDEX: SurnamesIndexResponse = {
  total_surnames: 4,
  total_people: 8,
  groups: [
    {
      letter: 'Л',
      items: [
        {
          canonical_surname: 'Левшиц',
          variants: ['Левшиц', 'Левшицъ', 'Levshits'],
          count: 3,
          earliest_year: 1888,
          latest_year: 1994,
        },
      ],
    },
    {
      letter: 'М',
      items: [
        {
          canonical_surname: 'Морозов',
          variants: ['Морозов', 'Морозова'],
          count: 1,
          earliest_year: 1915,
          latest_year: 1990,
        },
      ],
    },
    {
      letter: 'П',
      items: [
        {
          canonical_surname: 'Петров',
          variants: ['Петров', 'Петрова'],
          count: 3,
          earliest_year: 1912,
          latest_year: 1990,
        },
      ],
    },
    {
      letter: 'Щ',
      items: [
        {
          canonical_surname: 'Щеглов',
          variants: ['Щеглов', 'Щеглова'],
          count: 1,
          earliest_year: 1920,
          latest_year: 1998,
        },
      ],
    },
  ],
};

const CYRILLIC_ALPHABET = [
  'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'К',
  'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У', 'Ф',
  'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Э', 'Ю', 'Я',
];

const LATIN_ALPHABET = [
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
  'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
  'U', 'V', 'W', 'X', 'Y', 'Z',
];

export interface SurnamesPageProps {
  initialData?: SurnamesIndexResponse | null;
  onNavigate?: (path: string) => void;
}

export const SurnamesPage: React.FC<SurnamesPageProps> = ({
  initialData = null,
  onNavigate,
}) => {
  const { t, locale } = useI18n();
  const formatNum = (n: number) =>
    n.toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US');
  const [data, setData] = useState<SurnamesIndexResponse>(
    initialData ?? DEFAULT_SURNAMES_INDEX
  );
  const [selectedLetter, setSelectedLetter] = useState<string>('');
  const [searchFilter, setSearchFilter] = useState<string>('');

  const navigate = (path: string) => {
    if (onNavigate) {
      onNavigate(path);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', path);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  useEffect(() => {
    if (initialData) return;
    let cancelled = false;
    fetchSurnamesIndex()
      .then((res) => {
        if (!cancelled) {
          setData(res);
        }
      })
      .catch(() => {
        // Fallback to DEFAULT_SURNAMES_INDEX when offline
      });
    return () => {
      cancelled = true;
    };
  }, [initialData]);

  const activeLettersSet = useMemo(
    () => new Set(data.groups.map((g) => g.letter.toUpperCase())),
    [data.groups]
  );

  const filteredGroups: SurnameLetterGroupDto[] = useMemo(() => {
    return data.groups
      .filter(
        (group) =>
          !selectedLetter ||
          group.letter.toUpperCase() === selectedLetter.toUpperCase()
      )
      .map((group) => ({
        ...group,
        items: group.items.filter((item) => {
          if (!searchFilter.trim()) return true;
          const hay = `${item.canonical_surname} ${item.variants.join(' ')}`;
          return matchesCrossScript(hay, searchFilter.trim());
        }),
      }))
      .filter((group) => group.items.length > 0);
  }, [data.groups, selectedLetter, searchFilter]);

  return (
    <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-2.5">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => navigate('/people')}
              leftIcon={<ArrowLeft className="h-4 w-4" />}
              className="-ml-2"
            >
              {t('surnames.backToDirectory')}
            </Button>
          </div>
          <div className="flex flex-wrap items-center gap-2.5">
            <h1 className="font-sans text-2xl sm:text-3xl font-semibold tracking-tight text-primary">
              {t('surnames.title')}
            </h1>
            <Badge variant="brand">
              {t('surnames.totalBadge', {
                surnames: formatNum(data.total_surnames),
                people: formatNum(data.total_people),
              })}
            </Badge>
          </div>
          <p className="text-sm text-secondary">{t('surnames.subtitle')}</p>
        </div>

        <div className="w-full sm:w-72">
          <TextInput
            value={searchFilter}
            onChange={(e) => setSearchFilter(e.target.value)}
            leftIcon={<Search className="h-4 w-4 text-tertiary" />}
            placeholder={t('surnames.searchPlaceholder')}
            aria-label={t('surnames.searchPlaceholder')}
          />
        </div>
      </div>

      {/* Alphabetical Jump Bar (Cyrillic А–Я and Latin A–Z) */}
      <Card className="p-4 space-y-3">
        <div className="flex flex-wrap items-center gap-1.5" role="navigation" aria-label={t('surnames.alphabetNavAria')}>
          <Button
            variant={selectedLetter === '' ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setSelectedLetter('')}
            className="h-8 min-h-0 px-3 text-xs"
          >
            {t('surnames.allLetters')}
          </Button>

          {CYRILLIC_ALPHABET.map((letter) => {
            const hasEntries = activeLettersSet.has(letter);
            const isSelected = selectedLetter === letter;
            return (
              <Button
                key={`cyr-${letter}`}
                variant={isSelected ? 'primary' : 'ghost'}
                size="sm"
                disabled={!hasEntries && !isSelected}
                onClick={() =>
                  setSelectedLetter(isSelected ? '' : letter)
                }
                className="h-8 w-8 min-h-0 p-0 text-xs font-semibold"
              >
                {letter}
              </Button>
            );
          })}
        </div>

        <div className="flex flex-wrap items-center gap-1.5 border-t border-border pt-2.5">
          {LATIN_ALPHABET.map((letter) => {
            const hasEntries = activeLettersSet.has(letter);
            const isSelected = selectedLetter === letter;
            return (
              <Button
                key={`lat-${letter}`}
                variant={isSelected ? 'primary' : 'ghost'}
                size="sm"
                disabled={!hasEntries && !isSelected}
                onClick={() =>
                  setSelectedLetter(isSelected ? '' : letter)
                }
                className="h-7 w-7 min-h-0 p-0 text-xs font-medium"
              >
                {letter}
              </Button>
            );
          })}
        </div>
      </Card>

      {/* Surname Letter Groups & Cards */}
      {filteredGroups.length === 0 ? (
        <Card className="p-8">
          <EmptyState
            title={t('surnames.emptyTitle')}
            description={t('surnames.emptyDescription')}
          />
        </Card>
      ) : (
        <div className="space-y-8">
          {filteredGroups.map((group) => (
            <section
              key={group.letter}
              id={`surname-letter-${group.letter}`}
              className="space-y-3"
            >
              <div className="flex items-center gap-3 border-b border-border pb-2">
                <span className="inline-flex h-8 w-8 items-center justify-center rounded-xl bg-brand/15 font-serif text-lg font-bold text-brand">
                  {group.letter}
                </span>
                <span className="text-xs font-medium text-secondary">
                  {t('surnames.groupCount', { count: group.items.length })}
                </span>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {group.items.map((item) => {
                  const hasYearSpan =
                    item.earliest_year !== null || item.latest_year !== null;
                  const yearSpanText =
                    item.earliest_year !== null && item.latest_year !== null
                      ? item.earliest_year === item.latest_year
                        ? `${item.earliest_year}`
                        : `${item.earliest_year}–${item.latest_year}`
                      : `${item.earliest_year ?? item.latest_year ?? ''}`;

                  return (
                    <Card
                      key={item.canonical_surname}
                      className="p-4 flex flex-col justify-between gap-3 hover:border-brand/50 transition-colors"
                    >
                      <div className="space-y-1.5">
                        <div className="flex items-start justify-between gap-2">
                          <h2 className="font-serif text-lg font-semibold text-primary">
                            {item.canonical_surname}
                          </h2>
                          <Badge
                            variant="brand"
                            icon={<Users className="h-3 w-3" />}
                          >
                            {t('surnames.peopleCount', { count: item.count })}
                          </Badge>
                        </div>

                        {item.variants.length > 1 && (
                          <p className="text-xs text-secondary">
                            {t('surnames.variantsLabel')}:{' '}
                            <span className="font-medium text-primary">
                              {item.variants.join(' / ')}
                            </span>
                          </p>
                        )}

                        {hasYearSpan && (
                          <p className="inline-flex items-center gap-1.5 text-xs text-secondary">
                            <Calendar className="h-3.5 w-3.5 text-tertiary" />
                            <span>{yearSpanText}</span>
                          </p>
                        )}
                      </div>

                      <div className="pt-2 border-t border-border flex justify-end">
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() =>
                            navigate(
                              `/people?surname=${encodeURIComponent(
                                item.canonical_surname
                              )}`
                            )
                          }
                        >
                          {t('surnames.viewPeopleBtn')}
                        </Button>
                      </div>
                    </Card>
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      )}
    </div>
  );
};
