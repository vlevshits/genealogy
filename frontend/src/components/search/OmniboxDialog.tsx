import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Search } from 'lucide-react';
import {
  fetchGlobalSearch,
  SearchActionDto,
  SearchMediaResultDto,
  SearchPersonResultDto,
  SearchPlaceResultDto,
  SearchResultResponseDto,
} from '../../api/client';
import { Kbd, Spinner, TextInput } from '../ui';
import { SAMPLE_PEOPLE, SAMPLE_PLACES } from '../../lib/mock-data';
import { matchesCrossScript } from '../../lib/utils';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import {
  OmniboxResultsList,
  OmniboxSelectableItem,
  RecentPersonEntry,
} from './OmniboxResultsList';

const RECENT_PEOPLE_STORAGE_KEY = 'genealogy_recent_people';
const RECENT_SEARCHES_STORAGE_KEY = 'genealogy_recent_searches';
const MAX_RECENT_ITEMS = 5;

export function getRecentPeople(): RecentPersonEntry[] {
  try {
    if (typeof window === 'undefined' || !window.localStorage) return [];
    const raw = window.localStorage.getItem(RECENT_PEOPLE_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.slice(0, MAX_RECENT_ITEMS) : [];
  } catch {
    return [];
  }
}

export function recordRecentlyViewedPerson(entry: RecentPersonEntry): void {
  try {
    if (typeof window === 'undefined' || !window.localStorage) return;
    const current = getRecentPeople().filter((p) => p.id !== entry.id);
    const next = [entry, ...current].slice(0, MAX_RECENT_ITEMS);
    window.localStorage.setItem(
      RECENT_PEOPLE_STORAGE_KEY,
      JSON.stringify(next)
    );
  } catch {
    // Ignore storage errors
  }
}

export function getRecentSearches(): string[] {
  try {
    if (typeof window === 'undefined' || !window.localStorage) return [];
    const raw = window.localStorage.getItem(RECENT_SEARCHES_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.slice(0, MAX_RECENT_ITEMS) : [];
  } catch {
    return [];
  }
}

export function recordRecentSearch(queryText: string): void {
  const trimmed = queryText.trim();
  if (!trimmed) return;
  try {
    if (typeof window === 'undefined' || !window.localStorage) return;
    const current = getRecentSearches().filter(
      (q) => q.toLowerCase() !== trimmed.toLowerCase()
    );
    const next = [trimmed, ...current].slice(0, MAX_RECENT_ITEMS);
    window.localStorage.setItem(
      RECENT_SEARCHES_STORAGE_KEY,
      JSON.stringify(next)
    );
  } catch {
    // Ignore storage errors
  }
}

export function clearOmniboxRecentHistory(): void {
  try {
    if (typeof window === 'undefined' || !window.localStorage) return;
    window.localStorage.removeItem(RECENT_PEOPLE_STORAGE_KEY);
    window.localStorage.removeItem(RECENT_SEARCHES_STORAGE_KEY);
  } catch {
    // Ignore storage errors
  }
}

export interface OmniboxDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNavigate?: (path: string) => void;
  initialQuery?: string;
  initialResults?: SearchResultResponseDto | null;
  enableGlobalShortcut?: boolean;
}

export const OmniboxDialog: React.FC<OmniboxDialogProps> = ({
  open,
  onOpenChange,
  onNavigate,
  initialQuery = '',
  initialResults = null,
  enableGlobalShortcut = true,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const inputRef = useRef<HTMLInputElement | null>(null);

  const [query, setQuery] = useState<string>(initialQuery);
  const [debouncedQuery, setDebouncedQuery] = useState<string>(initialQuery);
  const [loading, setLoading] = useState<boolean>(false);
  const [results, setResults] = useState<SearchResultResponseDto | null>(
    initialResults
  );
  const [recentPeople, setRecentPeople] = useState<RecentPersonEntry[]>(() =>
    getRecentPeople()
  );
  const [recentSearches, setRecentSearches] = useState<string[]>(() =>
    getRecentSearches()
  );
  const [activeIndex, setActiveIndex] = useState<number>(0);

  const navigate = useCallback(
    (path: string) => {
      onOpenChange(false);
      if (onNavigate) {
        onNavigate(path);
      } else if (typeof window !== 'undefined') {
        window.history.pushState({}, '', path);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }
    },
    [onNavigate, onOpenChange]
  );

  // Global Cmd+K / Ctrl+K shortcut & Escape listener
  useEffect(() => {
    if (!enableGlobalShortcut) return;
    const onGlobalKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        onOpenChange(!open);
      } else if (e.key === 'Escape' && open) {
        onOpenChange(false);
      }
    };
    document.addEventListener('keydown', onGlobalKey);
    return () => document.removeEventListener('keydown', onGlobalKey);
  }, [enableGlobalShortcut, open, onOpenChange]);

  // Refresh recent state & focus input when opened
  useEffect(() => {
    if (open) {
      setRecentPeople(getRecentPeople());
      setRecentSearches(getRecentSearches());
      setActiveIndex(0);
      if (!initialQuery) {
        setQuery('');
        setDebouncedQuery('');
      }
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [open, initialQuery]);

  // 150ms debounce for search input
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(query);
    }, 150);
    return () => clearTimeout(timer);
  }, [query]);

  // Build role-aware fallback actions when offline or before response arrives
  const defaultActions: SearchActionDto[] = useMemo(() => {
    const role = user?.role ?? 'viewer';
    const base: SearchActionDto[] = [
      {
        id: 'nav-tree',
        label: t('omnibox.actionTreeLabel'),
        hint: t('omnibox.actionTreeHint'),
        path: '/tree',
        icon: 'git-branch',
        required_role: 'viewer',
      },
      {
        id: 'nav-people',
        label: t('omnibox.actionPeopleLabel'),
        hint: t('omnibox.actionPeopleHint'),
        path: '/people',
        icon: 'users',
        required_role: 'viewer',
      },
      {
        id: 'nav-surnames',
        label: t('omnibox.actionSurnamesLabel'),
        hint: t('omnibox.actionSurnamesHint'),
        path: '/people/surnames',
        icon: 'book-open',
        required_role: 'viewer',
      },
      {
        id: 'nav-places',
        label: t('omnibox.actionPlacesLabel'),
        hint: t('omnibox.actionPlacesHint'),
        path: '/places',
        icon: 'map-pin',
        required_role: 'viewer',
      },
    ];
    if (role === 'admin' || role === 'editor') {
      base.push({
        id: 'act-add-person',
        label: t('omnibox.actionAddPersonLabel'),
        hint: t('omnibox.actionAddPersonHint'),
        path: '/people?action=add',
        icon: 'user-plus',
        required_role: 'editor',
      });
    }
    if (role === 'admin') {
      base.push(
        {
          id: 'act-import-gedcom',
          label: t('omnibox.actionGedcomLabel'),
          hint: t('omnibox.actionGedcomHint'),
          path: '/settings/gedcom',
          icon: 'upload',
          required_role: 'admin',
        },
        {
          id: 'nav-settings',
          label: t('omnibox.actionSettingsLabel'),
          hint: t('omnibox.actionSettingsHint'),
          path: '/settings',
          icon: 'settings',
          required_role: 'admin',
        }
      );
    }
    return base;
  }, [t, user?.role]);

  // Fetch search results from backend API when open
  useEffect(() => {
    if (!open || initialResults) return;
    let cancelled = false;
    setLoading(true);
    fetchGlobalSearch({ q: debouncedQuery })
      .then((res) => {
        if (!cancelled) {
          setResults(res);
          setActiveIndex(0);
        }
      })
      .catch(() => {
        if (!cancelled) {
          // Offline fallback using sample dataset & cross-script matching
          const qTrim = debouncedQuery.trim();
          const fallbackPeople: SearchPersonResultDto[] = qTrim
            ? SAMPLE_PEOPLE.filter(
                (p) =>
                  !p.is_redacted &&
                  matchesCrossScript(
                    `${p.surname ?? ''} ${p.birth_surname ?? ''} ${p.given_name ?? ''} ${p.patronymic ?? ''} ${p.gid} ${p.birth_date ?? ''}`,
                    qTrim
                  )
              )
                .slice(0, 8)
                .map((p, idx) => ({
                  id: p.id,
                  gid: p.gid,
                  display_name: [p.surname, p.given_name, p.patronymic]
                    .filter(Boolean)
                    .join(' '),
                  given_name: p.given_name ?? '',
                  patronymic: p.patronymic ?? null,
                  surname: p.surname ?? '',
                  sex: p.sex ?? 'U',
                  birth_year: p.birth_date?.start?.y ?? null,
                  death_year: p.death_date?.start?.y ?? null,
                  birth_place: p.birth_place ?? null,
                  avatar_url: null,
                  is_living: Boolean(p.is_living),
                  is_private: false,
                  score: 100 - idx,
                }))
            : [];

          const fallbackPlaces: SearchPlaceResultDto[] = qTrim
            ? SAMPLE_PLACES.filter((pl) =>
                matchesCrossScript(
                  `${pl.name} ${pl.hierarchy_names?.join(' ') ?? ''} ${pl.gid}`,
                  qTrim
                )
              )
                .slice(0, 3)
                .map((pl) => ({
                  id: pl.id,
                  gid: pl.gid,
                  name: pl.name,
                  full_name: pl.hierarchy_names?.join(', ') || pl.name,
                  place_type: 'city',
                  events_count: pl.events_count ?? 0,
                  has_coordinates: true,
                }))
            : [];

          const fallbackActions = qTrim
            ? defaultActions.filter((a) =>
                matchesCrossScript(`${a.label} ${a.hint}`, qTrim)
              )
            : defaultActions;

          setResults({
            query: qTrim,
            people: fallbackPeople,
            places: fallbackPlaces,
            media: [],
            actions: fallbackActions,
            timing_ms: 1,
          });
          setActiveIndex(0);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [open, debouncedQuery, defaultActions, initialResults]);

  const people: SearchPersonResultDto[] = results?.people ?? [];
  const places: SearchPlaceResultDto[] = results?.places ?? [];
  const media: SearchMediaResultDto[] = results?.media ?? [];
  const actions: SearchActionDto[] = results?.actions ?? defaultActions;

  const selectableItems: OmniboxSelectableItem[] = useMemo(() => {
    const items: OmniboxSelectableItem[] = [];
    if (query.trim().length === 0) {
      for (const person of recentPeople) {
        items.push({
          kind: 'person',
          key: `recent-person-${person.id}`,
          person,
        });
      }
      for (const qText of recentSearches) {
        items.push({
          kind: 'recent-search',
          key: `recent-search-${qText}`,
          queryText: qText,
        });
      }
      for (const action of actions) {
        items.push({
          kind: 'action',
          key: `action-${action.id}`,
          action,
        });
      }
    } else {
      for (const person of people) {
        items.push({
          kind: 'person',
          key: `person-${person.id}`,
          person,
        });
      }
      for (const place of places) {
        items.push({
          kind: 'place',
          key: `place-${place.id}`,
          place,
        });
      }
      for (const m of media) {
        items.push({
          kind: 'media',
          key: `media-${m.id}`,
          media: m,
        });
      }
      for (const action of actions) {
        items.push({
          kind: 'action',
          key: `action-${action.id}`,
          action,
        });
      }
    }
    return items;
  }, [query, recentPeople, recentSearches, people, places, media, actions]);

  const handleSelectItem = useCallback(
    (item: OmniboxSelectableItem, focusInTree = false) => {
      if (query.trim()) {
        recordRecentSearch(query.trim());
      }
      if (item.kind === 'person') {
        recordRecentlyViewedPerson({
          id: item.person.id,
          gid: item.person.gid,
          display_name: item.person.display_name,
          sex: item.person.sex,
          birth_year: item.person.birth_year,
          death_year: item.person.death_year,
          birth_place: item.person.birth_place,
          avatar_url: item.person.avatar_url,
          is_living: item.person.is_living,
        });
        if (focusInTree) {
          navigate(`/tree?focus=${encodeURIComponent(item.person.id)}`);
        } else {
          navigate(`/persons/${encodeURIComponent(item.person.id)}`);
        }
        return;
      }
      if (item.kind === 'place') {
        navigate(`/places?selected=${encodeURIComponent(item.place.id)}`);
        return;
      }
      if (item.kind === 'media') {
        navigate(`/media?selected=${encodeURIComponent(item.media.id)}`);
        return;
      }
      if (item.kind === 'action') {
        const normalizedPath = item.action.path.startsWith('/admin/')
          ? item.action.path.replace('/admin/', '/settings/')
          : item.action.path;
        navigate(normalizedPath);
        return;
      }
      if (item.kind === 'recent-search') {
        setQuery(item.queryText);
        setDebouncedQuery(item.queryText);
        setActiveIndex(0);
      }
    },
    [navigate, query]
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    const count = selectableItems.length;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (count > 0) {
        setActiveIndex((prev) => (prev + 1) % count);
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (count > 0) {
        setActiveIndex((prev) => (prev - 1 + count) % count);
      }
    } else if (e.key === 'Enter' && selectableItems[activeIndex]) {
      e.preventDefault();
      handleSelectItem(selectableItems[activeIndex], e.shiftKey);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onOpenChange(false);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-14 sm:pt-20 px-4">
      <div
        aria-hidden="true"
        onClick={() => onOpenChange(false)}
        className="fixed inset-0 bg-primary/40 backdrop-blur-xs"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t('omnibox.title')}
        onKeyDown={handleKeyDown}
        className="relative z-10 w-full max-w-2xl overflow-hidden rounded-2xl border border-border bg-surface shadow-2xl"
      >
        {/* Search Input Header */}
        <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
          <div className="flex-1">
            <TextInput
              ref={inputRef}
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setActiveIndex(0);
              }}
              leftIcon={<Search className="h-4 w-4 text-tertiary" />}
              placeholder={t('omnibox.placeholder')}
              aria-label={t('omnibox.placeholder')}
              className="border-transparent bg-transparent focus-within:ring-0"
            />
          </div>
          {loading && <Spinner size="sm" />}
          <Kbd>Esc</Kbd>
        </div>

        {/* Grouped Results & Empty State */}
        <OmniboxResultsList
          query={query}
          loading={loading}
          people={people}
          places={places}
          media={media}
          actions={actions}
          recentPeople={recentPeople}
          recentSearches={recentSearches}
          selectableItems={selectableItems}
          activeIndex={activeIndex}
          onActiveIndexChange={setActiveIndex}
          onSelectItem={handleSelectItem}
          onClearRecent={() => {
            clearOmniboxRecentHistory();
            setRecentPeople([]);
            setRecentSearches([]);
          }}
        />

        {/* Keyboard Shortcuts Footer */}
        <div className="flex flex-wrap items-center justify-between gap-2 border-t border-border bg-surface-muted/50 px-4 py-2 text-[11px] text-tertiary">
          <div className="flex flex-wrap items-center gap-3">
            <span className="inline-flex items-center gap-1">
              <Kbd>↑↓</Kbd>
              <span>{t('omnibox.hintNavigate')}</span>
            </span>
            <span className="inline-flex items-center gap-1">
              <Kbd>↵</Kbd>
              <span>{t('omnibox.hintSelect')}</span>
            </span>
            <span className="inline-flex items-center gap-1">
              <Kbd>⇧↵</Kbd>
              <span>{t('omnibox.hintFocusTree')}</span>
            </span>
            <span className="inline-flex items-center gap-1">
              <Kbd>Esc</Kbd>
              <span>{t('omnibox.hintClose')}</span>
            </span>
          </div>
          {results && query.trim().length > 0 && (
            <span className="font-mono text-[10px] text-tertiary">
              {results.timing_ms} ms
            </span>
          )}
        </div>
      </div>
    </div>
  );
};
