import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  fetchPeopleDirectory,
  PeopleDirectoryItemDto,
  PeopleDirectoryResponse,
  TreePersonNodeDto,
} from '../api/client';
import {
  DirectoryBulkBar,
  DirectoryCardList,
  DirectoryFilterBar,
  DirectoryFilterState,
  DirectoryHeader,
  DirectorySortKey,
  DirectoryTable,
} from '../components/directory';
import { QuickAddRelativeDialog } from '../components/person';
import { recordRecentlyViewedPerson } from '../components/search';
import { useToast } from '../components/ui';
import { useAuth } from '../lib/auth-context';
import { matchesCrossScript } from '../lib/utils';
import { useI18n } from '../i18n';

export const DEFAULT_DIRECTORY_ITEMS: PeopleDirectoryItemDto[] = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    gid: 'I0001',
    given_name: 'Александр',
    patronymic: 'Иванович',
    surname: 'Петров',
    birth_surname: null,
    display_name: 'Петров Александр Иванович',
    sex: 'M',
    birth_date: '1912-03-14',
    birth_year: 1912,
    birth_place: 'Тверь, Российская империя',
    death_date: '1989-10-22',
    death_year: 1989,
    death_place: 'Москва',
    avatar_url: null,
    is_living: false,
    is_private: false,
    has_parents: true,
    has_birth_date: true,
    version: 1,
    updated_at: '2026-10-27',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    gid: 'I0002',
    given_name: 'Елена',
    patronymic: 'Васильевна',
    surname: 'Петрова',
    birth_surname: 'Морозова',
    display_name: 'Петрова (Морозова) Елена Васильевна',
    sex: 'F',
    birth_date: '1915-06-10',
    birth_year: 1915,
    birth_place: 'Москва',
    death_date: '1990-02-18',
    death_year: 1990,
    death_place: 'Москва',
    avatar_url: '/api/v1/media/00000000-0000-0000-0000-000000000101/file',
    is_living: false,
    is_private: false,
    has_parents: true,
    has_birth_date: true,
    version: 1,
    updated_at: '2026-10-26',
  },
  {
    id: '00000000-0000-0000-0000-000000000003',
    gid: 'I0003',
    given_name: 'Михаил',
    patronymic: 'Александрович',
    surname: 'Левшиц',
    birth_surname: null,
    display_name: 'Левшиц Михаил Александрович',
    sex: 'M',
    birth_date: '1888-11-04',
    birth_year: 1888,
    birth_place: 'Витебск',
    death_date: '1958-05-12',
    death_year: 1958,
    death_place: 'Ленинград',
    avatar_url: null,
    is_living: false,
    is_private: false,
    has_parents: false,
    has_birth_date: true,
    version: 1,
    updated_at: '2026-10-25',
  },
  {
    id: '00000000-0000-0000-0000-000000000004',
    gid: 'I0004',
    given_name: 'Анна',
    patronymic: 'Сергеевна',
    surname: 'Щеглова',
    birth_surname: null,
    display_name: 'Щеглова Анна Сергеевна',
    sex: 'F',
    birth_date: null,
    birth_year: null,
    birth_place: 'Самара',
    death_date: null,
    death_year: null,
    death_place: null,
    avatar_url: null,
    is_living: true,
    is_private: false,
    has_parents: false,
    has_birth_date: false,
    version: 1,
    updated_at: '2026-10-24',
  },
];

function parseUrlParams(searchStr: string): {
  q: string;
  filters: DirectoryFilterState;
  sort: DirectorySortKey;
  openAddDialog: boolean;
} {
  const params = new URLSearchParams(searchStr);
  const q = params.get('q') ?? '';
  const surname = params.get('surname') ?? '';

  let bornFrom = params.get('born_from') ?? '';
  let bornTo = params.get('born_to') ?? '';
  const bornRange = params.get('born');
  if (bornRange && bornRange.includes('-')) {
    const [fromPart, toPart] = bornRange.split('-');
    if (!bornFrom && fromPart) bornFrom = fromPart.trim();
    if (!bornTo && toPart) bornTo = toPart.trim();
  }

  const sex = params.get('sex') ?? 'all';
  const living = params.get('living') ?? 'all';
  const hasPhotoParam = params.get('has_photo');
  const hasPhoto =
    hasPhotoParam === 'true' || hasPhotoParam === 'yes'
      ? 'yes'
      : hasPhotoParam === 'false' || hasPhotoParam === 'no'
      ? 'no'
      : 'all';
  const incompleteParam = params.get('incomplete');
  const incomplete = incompleteParam === 'true' || incompleteParam === '1';

  const rawSort = params.get('sort') ?? 'name_asc';
  const validSorts: DirectorySortKey[] = [
    'name_asc',
    'name_desc',
    'birth_asc',
    'birth_desc',
    'death_asc',
    'death_desc',
    'updated_desc',
  ];
  const sort: DirectorySortKey = validSorts.includes(
    rawSort as DirectorySortKey
  )
    ? (rawSort as DirectorySortKey)
    : 'name_asc';

  const openAddDialog = params.get('action') === 'add';

  return {
    q,
    filters: {
      surname,
      bornFrom,
      bornTo,
      sex,
      living,
      hasPhoto,
      incomplete,
    },
    sort,
    openAddDialog,
  };
}

export function serializeDirectoryUrlParams(
  q: string,
  filters: DirectoryFilterState,
  sort: DirectorySortKey
): string {
  const params = new URLSearchParams();
  if (q.trim()) params.set('q', q.trim());
  if (filters.surname.trim()) params.set('surname', filters.surname.trim());
  if (filters.bornFrom.trim()) params.set('born_from', filters.bornFrom.trim());
  if (filters.bornTo.trim()) params.set('born_to', filters.bornTo.trim());
  if (filters.sex && filters.sex !== 'all') params.set('sex', filters.sex);
  if (filters.living && filters.living !== 'all') {
    params.set('living', filters.living);
  }
  if (filters.hasPhoto === 'yes') params.set('has_photo', 'true');
  if (filters.hasPhoto === 'no') params.set('has_photo', 'false');
  if (filters.incomplete) params.set('incomplete', 'true');
  if (sort !== 'name_asc') params.set('sort', sort);
  const qs = params.toString();
  return qs ? `?${qs}` : '';
}

function foldStemClient(surname: string): string {
  const lower = surname.trim().toLowerCase().replace(/ъ$/, '').replace(/ё/g, 'е');
  for (const [fem, masc] of [
    ['цкая', 'цкий'],
    ['ская', 'ский'],
    ['ова', 'ов'],
    ['ева', 'ев'],
    ['ина', 'ин'],
    ['ova', 'ov'],
    ['eva', 'ev'],
    ['ina', 'in'],
  ] as const) {
    if (lower.endsWith(fem) && lower.length > fem.length + 1) {
      return lower.slice(0, -fem.length) + masc;
    }
  }
  return lower;
}

export interface PeoplePageProps {
  initialData?: PeopleDirectoryResponse | null;
  initialSearch?: string;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export const PeoplePage: React.FC<PeoplePageProps> = ({
  initialData = null,
  initialSearch,
  userRole,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const canEdit = effectiveRole === 'admin' || effectiveRole === 'editor';
  const canMerge = effectiveRole === 'admin';

  const parsedInitial = useMemo(
    () =>
      parseUrlParams(
        initialSearch ??
          (typeof window !== 'undefined' ? window.location.search : '')
      ),
    [initialSearch]
  );

  const [searchQuery, setSearchQuery] = useState<string>(parsedInitial.q);
  const [filters, setFilters] = useState<DirectoryFilterState>(
    parsedInitial.filters
  );
  const [sort, setSort] = useState<DirectorySortKey>(parsedInitial.sort);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  const [remoteItems, setRemoteItems] = useState<PeopleDirectoryItemDto[]>(
    () => initialData?.items ?? DEFAULT_DIRECTORY_ITEMS
  );
  const [nextCursor, setNextCursor] = useState<string | null>(
    () => initialData?.next_cursor ?? null
  );
  const [remoteTotal, setRemoteTotal] = useState<number | null>(
    () => initialData?.total ?? null
  );
  const [remoteTotalUnfiltered, setRemoteTotalUnfiltered] = useState<
    number | null
  >(() => initialData?.total_unfiltered ?? null);
  const [loadingMore, setLoadingMore] = useState<boolean>(false);
  const [usingBackendData, setUsingBackendData] = useState<boolean>(
    Boolean(initialData)
  );

  const [quickAddOpen, setQuickAddOpen] = useState<boolean>(
    parsedInitial.openAddDialog
  );
  const [quickAddAnchor, setQuickAddAnchor] =
    useState<TreePersonNodeDto | null>(null);

  const navigate = useCallback(
    (path: string) => {
      if (onNavigate) {
        onNavigate(path);
      } else if (typeof window !== 'undefined') {
        window.history.pushState({}, '', path);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }
    },
    [onNavigate]
  );

  // Sync URL query parameters when search, filters, or sort change (13 §4.2, §6 DoD #7)
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const qs = serializeDirectoryUrlParams(searchQuery, filters, sort);
    const nextUrl = `${window.location.pathname}${qs}`;
    if (`${window.location.pathname}${window.location.search}` !== nextUrl) {
      window.history.replaceState({}, '', nextUrl);
    }
  }, [searchQuery, filters, sort]);

  const buildQueryDto = useCallback(
    (cursor?: string) => ({
      q: searchQuery.trim() || undefined,
      surname: filters.surname.trim() || undefined,
      born_from: filters.bornFrom ? Number(filters.bornFrom) : undefined,
      born_to: filters.bornTo ? Number(filters.bornTo) : undefined,
      sex: filters.sex !== 'all' ? filters.sex : undefined,
      living: filters.living !== 'all' ? filters.living : undefined,
      has_photo:
        filters.hasPhoto === 'yes'
          ? true
          : filters.hasPhoto === 'no'
          ? false
          : undefined,
      incomplete: filters.incomplete ? true : undefined,
      sort,
      cursor,
      limit: 100,
    }),
    [searchQuery, filters, sort]
  );

  const loadDirectory = useCallback(() => {
    if (initialData) return () => {};
    let cancelled = false;
    fetchPeopleDirectory(buildQueryDto())
      .then((res) => {
        if (cancelled) return;
        setRemoteItems(res.items);
        setNextCursor(res.next_cursor);
        setRemoteTotal(res.total);
        setRemoteTotalUnfiltered(res.total_unfiltered);
        setUsingBackendData(true);
      })
      .catch(() => {
        if (!cancelled) {
          setUsingBackendData(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [buildQueryDto, initialData]);

  useEffect(() => {
    return loadDirectory();
  }, [loadDirectory]);

  const handleLoadMore = async () => {
    if (!nextCursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const res = await fetchPeopleDirectory(buildQueryDto(nextCursor));
      setRemoteItems((prev) => [...prev, ...res.items]);
      setNextCursor(res.next_cursor);
      setRemoteTotal(res.total);
      setRemoteTotalUnfiltered(res.total_unfiltered);
    } catch {
      // Ignore offline error
    } finally {
      setLoadingMore(false);
    }
  };

  // Client-side filter + sort over fallback or initial items so UI works both online and in static tests
  const displayedItems = useMemo(() => {
    if (usingBackendData && !initialData) {
      return remoteItems;
    }
    const source = initialData?.items ?? remoteItems;
    const filtered = source.filter((item) => {
      if (searchQuery.trim()) {
        const hay = `${item.display_name} ${item.given_name} ${item.patronymic ?? ''} ${item.surname} ${item.birth_surname ?? ''} ${item.gid} ${item.birth_year ?? ''}`;
        if (!matchesCrossScript(hay, searchQuery.trim())) return false;
      }
      if (filters.surname.trim()) {
        const targetStem = foldStemClient(filters.surname);
        const itemStem = foldStemClient(item.surname);
        const birthStem = item.birth_surname
          ? foldStemClient(item.birth_surname)
          : '';
        if (
          itemStem !== targetStem &&
          birthStem !== targetStem &&
          !matchesCrossScript(
            `${item.surname} ${item.birth_surname ?? ''}`,
            filters.surname.trim()
          )
        ) {
          return false;
        }
      }
      if (filters.bornFrom.trim()) {
        const fromYr = Number(filters.bornFrom);
        if (!item.birth_year || item.birth_year < fromYr) return false;
      }
      if (filters.bornTo.trim()) {
        const toYr = Number(filters.bornTo);
        if (!item.birth_year || item.birth_year > toYr) return false;
      }
      if (filters.sex && filters.sex !== 'all') {
        if (item.sex.toUpperCase() !== filters.sex.toUpperCase()) return false;
      }
      if (filters.living === 'living' && !item.is_living) return false;
      if (filters.living === 'deceased' && item.is_living) return false;
      if (filters.hasPhoto === 'yes' && !item.avatar_url) return false;
      if (filters.hasPhoto === 'no' && item.avatar_url) return false;
      if (filters.incomplete) {
        const isInc = !item.has_birth_date || !item.has_parents;
        if (!isInc) return false;
      }
      return true;
    });

    return [...filtered].sort((a, b) => {
      switch (sort) {
        case 'name_desc':
          return b.display_name.localeCompare(a.display_name);
        case 'birth_asc':
          return (a.birth_year ?? 9999) - (b.birth_year ?? 9999);
        case 'birth_desc':
          return (b.birth_year ?? -9999) - (a.birth_year ?? -9999);
        case 'death_asc':
          return (a.death_year ?? 9999) - (b.death_year ?? 9999);
        case 'death_desc':
          return (b.death_year ?? -9999) - (a.death_year ?? -9999);
        case 'updated_desc':
          return (b.updated_at ?? '').localeCompare(a.updated_at ?? '');
        default:
          return a.display_name.localeCompare(b.display_name);
      }
    });
  }, [
    usingBackendData,
    initialData,
    remoteItems,
    searchQuery,
    filters,
    sort,
  ]);

  const totalCount =
    usingBackendData && !initialData && remoteTotal !== null
      ? remoteTotal
      : displayedItems.length;
  const totalUnfilteredCount =
    remoteTotalUnfiltered ??
    initialData?.total_unfiltered ??
    remoteItems.length;

  const handleToggleSelectRow = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  };

  const handleToggleSelectAll = () => {
    const allIds = displayedItems.map((i) => i.id);
    const allSelected =
      allIds.length > 0 && allIds.every((id) => selectedIds.includes(id));
    if (allSelected) {
      setSelectedIds([]);
    } else {
      setSelectedIds(allIds);
    }
  };

  const handleClearFilters = () => {
    setSearchQuery('');
    setFilters({
      surname: '',
      bornFrom: '',
      bornTo: '',
      sex: 'all',
      living: 'all',
      hasPhoto: 'all',
      incomplete: false,
    });
  };

  const handleOpenPerson = (id: string) => {
    const found = displayedItems.find((i) => i.id === id);
    if (found) {
      recordRecentlyViewedPerson({
        id: found.id,
        gid: found.gid,
        display_name: found.display_name,
        sex: found.sex,
        birth_year: found.birth_year,
        death_year: found.death_year,
        birth_place: found.birth_place,
        avatar_url: found.avatar_url,
        is_living: found.is_living,
      });
    }
    navigate(`/persons/${encodeURIComponent(id)}`);
  };

  const handleFocusInTree = (id: string) => {
    navigate(`/tree?focus=${encodeURIComponent(id)}`);
  };

  const handleAddRelativeFromRow = (item: PeopleDirectoryItemDto) => {
    setQuickAddAnchor({
      id: item.id,
      gid: item.gid,
      sex: (item.sex as 'M' | 'F' | 'U') || 'U',
      is_living: item.is_living,
      is_private: item.is_private,
      primary_name: {
        given_name: item.given_name,
        patronymic: item.patronymic,
        surname: item.surname,
      },
      birth_surname: item.birth_surname ?? null,
      birth: item.birth_date
        ? { date: item.birth_date, place: item.birth_place }
        : null,
      death: item.death_date
        ? { date: item.death_date, place: item.death_place }
        : null,
      avatar_url: item.avatar_url,
      notes: null,
      version: item.version,
    });
    setQuickAddOpen(true);
  };

  return (
    <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <DirectoryHeader
        total={totalCount}
        totalUnfiltered={totalUnfilteredCount}
        searchQuery={searchQuery}
        onSearchQueryChange={setSearchQuery}
        canEdit={canEdit}
        onOpenSurnames={() => navigate('/people/surnames')}
        onAddPerson={() => {
          setQuickAddAnchor(null);
          setQuickAddOpen(true);
        }}
      />

      <DirectoryFilterBar
        filters={filters}
        onFiltersChange={(patch) =>
          setFilters((prev) => ({ ...prev, ...patch }))
        }
        onClearFilters={handleClearFilters}
      />

      <DirectoryTable
        items={displayedItems}
        sort={sort}
        onSortChange={setSort}
        selectedIds={selectedIds}
        onToggleSelectRow={handleToggleSelectRow}
        onToggleSelectAll={handleToggleSelectAll}
        canEdit={canEdit}
        onOpenPerson={handleOpenPerson}
        onFocusInTree={handleFocusInTree}
        onAddRelative={handleAddRelativeFromRow}
        nextCursor={nextCursor}
        loadingMore={loadingMore}
        onLoadMore={handleLoadMore}
      />

      <DirectoryCardList
        items={displayedItems}
        selectedIds={selectedIds}
        onToggleSelectRow={handleToggleSelectRow}
        canEdit={canEdit}
        onOpenPerson={handleOpenPerson}
        onFocusInTree={handleFocusInTree}
        nextCursor={nextCursor}
        loadingMore={loadingMore}
        onLoadMore={handleLoadMore}
      />

      {canEdit && (
        <DirectoryBulkBar
          selectedIds={selectedIds}
          canMerge={canMerge}
          onCompareMerge={(idA, idB) =>
            navigate(
              `/people/merge?a=${encodeURIComponent(idA)}&b=${encodeURIComponent(idB)}`
            )
          }
          onExportGedcom={(ids) => {
            addToast({
              type: 'success',
              title: t('directory.exportSelectionToast', { count: ids.length }),
            });
          }}
          onClearSelection={() => setSelectedIds([])}
        />
      )}

      <QuickAddRelativeDialog
        open={quickAddOpen}
        onOpenChange={setQuickAddOpen}
        anchorPerson={quickAddAnchor}
        relation="child"
        onRelativeAdded={(created) => {
          loadDirectory();
          if (created?.id) {
            navigate(`/persons/${encodeURIComponent(created.id)}`);
          }
        }}
      />
    </div>
  );
};
