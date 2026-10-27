import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  fetchMediaArchive,
  fetchPeopleDirectory,
  MediaArchiveResponse,
  MediaAssetSummaryDto,
} from '../api/client';
import {
  FaceTagCandidatePerson,
  MediaCategoryFilter,
  MediaFilterBar,
  MediaFilterState,
  MediaLightboxStudio,
  MediaPersonOption,
  MediaUploadDialog,
  MediaWall,
} from '../components/media';
import { useAuth } from '../lib/auth-context';
import { matchesCrossScript } from '../lib/utils';

export const DEFAULT_FACE_CANDIDATES: FaceTagCandidatePerson[] = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    gid: 'I0001',
    displayName: 'Левшиц Михаил Александрович',
    birthYear: 1912,
    deathYear: 1989,
    isLiving: false,
    avatarUrl: '/api/v1/media/00000000-0000-0000-0000-000000000101/file',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    gid: 'I0002',
    displayName: 'Петров Александр Иванович',
    birthYear: 1912,
    deathYear: 1989,
    isLiving: false,
    avatarUrl: null,
  },
  {
    id: 'c7a8b9d0-1234-5678-90ab-cdef12345678',
    gid: 'I0042',
    displayName: 'Левшиц Елена Александровна',
    birthYear: 1994,
    deathYear: null,
    isLiving: true,
    avatarUrl: '/api/v1/media/00000000-0000-0000-0000-000000000102/file',
  },
];

export const DEFAULT_MEDIA_ARCHIVE_ITEMS: MediaAssetSummaryDto[] = [
  {
    id: '00000000-0000-0000-0000-000000000101',
    gid: 'M0001',
    title: 'Mikhail Levshits Archival Portrait (1955)',
    mime_type: 'image/webp',
    file_url: '/api/v1/media/00000000-0000-0000-0000-000000000101/file',
    width: 1200,
    height: 900,
    description: 'Studio portrait taken in Leningrad in 1955.',
    date_taken: '1955',
    date_sort: 19550000,
    place_id: '00000000-0000-0000-0000-000000000201',
    place_full_name: 'Saint Petersburg, Russia',
    is_private: false,
    only_living_persons: false,
    linked_person_ids: [
      '00000000-0000-0000-0000-000000000001',
      'c7a8b9d0-1234-5678-90ab-cdef12345678',
    ],
    face_tags: [
      {
        id: 'ft-01',
        person_id: '00000000-0000-0000-0000-000000000001',
        person_name: 'Левшиц Михаил Александрович',
        is_living: false,
        x: 0.25,
        y: 0.3,
        width: 0.2,
        height: 0.25,
        is_primary_avatar: true,
      },
      {
        id: 'ft-02',
        person_id: 'c7a8b9d0-1234-5678-90ab-cdef12345678',
        person_name: 'Левшиц Елена Александровна',
        is_living: true,
        x: 0.6,
        y: 0.35,
        width: 0.18,
        height: 0.22,
        is_primary_avatar: false,
      },
    ],
    version: 1,
  },
  {
    id: '00000000-0000-0000-0000-000000000102',
    gid: 'M0002',
    title: 'Metrical Book Birth Record (1912)',
    mime_type: 'application/pdf',
    file_url: '/api/v1/media/00000000-0000-0000-0000-000000000102/file',
    description: 'Archival parish register scan from Vitebsk.',
    date_taken: '1912-03-19',
    date_sort: 19120319,
    place_full_name: 'Vitebsk, Belarus',
    is_private: false,
    only_living_persons: false,
    linked_person_ids: ['00000000-0000-0000-0000-000000000001'],
    face_tags: [],
    version: 1,
  },
  {
    id: '00000000-0000-0000-0000-000000000103',
    gid: 'M0003',
    title: 'Unidentified Family Gathering in Tver (1941)',
    mime_type: 'image/jpeg',
    file_url: '/api/v1/media/00000000-0000-0000-0000-000000000103/file',
    width: 1200,
    height: 1500,
    description: 'Photograph found in family album before evacuation.',
    date_taken: 'Circa Sep 1941',
    date_sort: 19410900,
    place_full_name: 'Tver, Russia',
    is_private: false,
    only_living_persons: false,
    linked_person_ids: [],
    face_tags: [],
    version: 1,
  },
];

function extractAssetYear(asset: MediaAssetSummaryDto): number | null {
  if (asset.date_sort && asset.date_sort >= 10000000) {
    return Math.floor(asset.date_sort / 10000);
  }
  const combined = `${asset.date_taken ?? ''} ${asset.title}`;
  const match = combined.match(/\b(1[5-9]\d{2}|20\d{2})\b/);
  return match ? Number(match[1]) : null;
}

function parseMediaUrlParams(searchStr: string): {
  filters: MediaFilterState;
  selectedId: string | null;
} {
  const params = new URLSearchParams(searchStr);
  const q = params.get('q') ?? '';
  const personId = params.get('person_id') ?? '';
  const decade = params.get('decade') ?? '';
  const rawType = (params.get('type') ?? 'all').toLowerCase();
  const type: MediaCategoryFilter =
    rawType === 'photos' || rawType === 'documents' || rawType === 'audio'
      ? rawType
      : 'all';
  const untaggedParam = params.get('untagged');
  const untagged = untaggedParam === 'true' || untaggedParam === '1';
  const selectedId = params.get('selected');

  return {
    filters: {
      q,
      personId,
      decade,
      type,
      untagged,
    },
    selectedId,
  };
}

export function serializeMediaUrlParams(filters: MediaFilterState): string {
  const params = new URLSearchParams();
  if (filters.q.trim()) params.set('q', filters.q.trim());
  if (filters.personId.trim()) params.set('person_id', filters.personId.trim());
  if (filters.decade.trim()) params.set('decade', filters.decade.trim());
  if (filters.type !== 'all') params.set('type', filters.type);
  if (filters.untagged) params.set('untagged', 'true');
  const qs = params.toString();
  return qs ? `?${qs}` : '';
}

export interface MediaPageProps {
  initialData?: MediaArchiveResponse | null;
  initialSearch?: string;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export const MediaPage: React.FC<MediaPageProps> = ({
  initialData = null,
  initialSearch,
  userRole,
  onNavigate,
}) => {
  const { user } = useAuth();
  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const canEdit = effectiveRole === 'admin' || effectiveRole === 'editor';

  const parsedInitial = useMemo(
    () =>
      parseMediaUrlParams(
        initialSearch ??
          (typeof window !== 'undefined' ? window.location.search : '')
      ),
    [initialSearch]
  );

  const [filters, setFilters] = useState<MediaFilterState>(
    parsedInitial.filters
  );
  const [remoteItems, setRemoteItems] = useState<MediaAssetSummaryDto[]>(
    () => initialData?.items ?? DEFAULT_MEDIA_ARCHIVE_ITEMS
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
  const [usingBackendData, setUsingBackendData] = useState<boolean>(
    Boolean(initialData)
  );
  const [loadingMore, setLoadingMore] = useState<boolean>(false);

  const [candidates, setCandidates] = useState<FaceTagCandidatePerson[]>(
    DEFAULT_FACE_CANDIDATES
  );
  const [uploadOpen, setUploadOpen] = useState<boolean>(false);
  const [activeAssetId, setActiveAssetId] = useState<string | null>(
    parsedInitial.selectedId
  );

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

  // Sync URL query params
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const qs = serializeMediaUrlParams(filters);
    const nextUrl = `${window.location.pathname}${qs}`;
    if (`${window.location.pathname}${window.location.search}` !== nextUrl) {
      window.history.replaceState({}, '', nextUrl);
    }
  }, [filters]);

  const buildQueryDto = useCallback(
    (cursor?: string) => ({
      q: filters.q.trim() || undefined,
      person_id: filters.personId || undefined,
      decade: filters.decade || undefined,
      type: filters.type !== 'all' ? filters.type : undefined,
      untagged: filters.untagged ? true : undefined,
      cursor,
      limit: 50,
    }),
    [filters]
  );

  const loadMedia = useCallback(() => {
    if (initialData) return;
    fetchMediaArchive(buildQueryDto())
      .then((res) => {
        setRemoteItems(res.items);
        setNextCursor(res.next_cursor);
        setRemoteTotal(res.total);
        setRemoteTotalUnfiltered(res.total_unfiltered);
        setUsingBackendData(true);
      })
      .catch(() => {
        setUsingBackendData(false);
      });
  }, [buildQueryDto, initialData]);

  useEffect(() => {
    loadMedia();
  }, [loadMedia]);

  useEffect(() => {
    fetchPeopleDirectory({ limit: 100 })
      .then((res) => {
        if (res.items.length > 0) {
          setCandidates(
            res.items.map((p) => ({
              id: p.id,
              gid: p.gid,
              displayName: p.display_name,
              birthYear: p.birth_year,
              deathYear: p.death_year,
              isLiving: p.is_living,
              avatarUrl: p.avatar_url,
            }))
          );
        }
      })
      .catch(() => {
        // Keep DEFAULT_FACE_CANDIDATES when offline
      });
  }, []);

  const displayedItems = useMemo(() => {
    if (usingBackendData && !initialData) {
      return remoteItems;
    }
    const source = initialData?.items ?? remoteItems;
    return source.filter((item) => {
      if (filters.personId) {
        const matchesPerson =
          item.linked_person_ids.includes(filters.personId) ||
          item.face_tags.some((tItem) => tItem.person_id === filters.personId);
        if (!matchesPerson) return false;
      }
      if (filters.decade) {
        const decStart = parseInt(filters.decade, 10);
        if (!Number.isNaN(decStart)) {
          const yr = extractAssetYear(item);
          if (!yr || yr < decStart || yr > decStart + 9) return false;
        }
      }
      if (filters.type !== 'all') {
        if (
          filters.type === 'photos' &&
          !item.mime_type.startsWith('image/')
        ) {
          return false;
        }
        if (
          filters.type === 'documents' &&
          item.mime_type !== 'application/pdf' &&
          !item.mime_type.startsWith('text/')
        ) {
          return false;
        }
        if (filters.type === 'audio' && !item.mime_type.startsWith('audio/')) {
          return false;
        }
      }
      if (filters.untagged) {
        const isUntagged =
          item.face_tags.length === 0 && item.linked_person_ids.length === 0;
        if (!isUntagged) return false;
      }
      if (filters.q.trim()) {
        const hay = `${item.title} ${item.gid} ${item.description ?? ''} ${item.place_full_name ?? ''} ${item.face_tags.map((f) => f.person_name).join(' ')}`;
        if (!matchesCrossScript(hay, filters.q.trim())) return false;
      }
      return true;
    });
  }, [usingBackendData, initialData, remoteItems, filters]);

  const totalCount =
    usingBackendData && !initialData && remoteTotal !== null
      ? remoteTotal
      : displayedItems.length;
  const totalUnfilteredCount =
    remoteTotalUnfiltered ??
    initialData?.total_unfiltered ??
    remoteItems.length;

  const peopleOptions: MediaPersonOption[] = useMemo(
    () =>
      candidates.map((c) => ({
        id: c.id,
        displayName: c.displayName,
        gid: c.gid,
      })),
    [candidates]
  );

  const activeAsset = useMemo(
    () =>
      activeAssetId
        ? displayedItems.find((i) => i.id === activeAssetId) ??
          remoteItems.find((i) => i.id === activeAssetId) ??
          null
        : null,
    [activeAssetId, displayedItems, remoteItems]
  );

  const activeIndex = useMemo(
    () =>
      activeAsset
        ? displayedItems.findIndex((i) => i.id === activeAsset.id)
        : -1,
    [activeAsset, displayedItems]
  );

  const handleLoadMore = async () => {
    if (!nextCursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const res = await fetchMediaArchive(buildQueryDto(nextCursor));
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

  return (
    <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <MediaFilterBar
        total={totalCount}
        totalUnfiltered={totalUnfilteredCount}
        filters={filters}
        peopleOptions={peopleOptions}
        canEdit={canEdit}
        onFiltersChange={(patch) =>
          setFilters((prev) => ({ ...prev, ...patch }))
        }
        onClearFilters={() =>
          setFilters({
            q: '',
            personId: '',
            decade: '',
            type: 'all',
            untagged: false,
          })
        }
        onOpenUpload={() => setUploadOpen(true)}
      />

      <MediaWall
        items={displayedItems}
        onSelectAsset={(asset) => setActiveAssetId(asset.id)}
        nextCursor={nextCursor}
        loadingMore={loadingMore}
        onLoadMore={handleLoadMore}
      />

      <MediaUploadDialog
        open={uploadOpen}
        onOpenChange={setUploadOpen}
        onUploaded={(created) => {
          setRemoteItems((prev) => [...created, ...prev]);
          loadMedia();
        }}
      />

      <MediaLightboxStudio
        open={Boolean(activeAsset)}
        onOpenChange={(open) => {
          if (!open) setActiveAssetId(null);
        }}
        asset={activeAsset}
        canEdit={canEdit}
        candidates={candidates}
        onAssetUpdated={(updated) => {
          setRemoteItems((prev) =>
            prev.map((item) => (item.id === updated.id ? updated : item))
          );
        }}
        onAssetDeleted={(deletedId) => {
          setRemoteItems((prev) =>
            prev.filter((item) => item.id !== deletedId)
          );
          setActiveAssetId(null);
        }}
        onAvatarSynchronized={(personId, avatarUrl) => {
          setCandidates((prev) =>
            prev.map((c) =>
              c.id === personId ? { ...c, avatarUrl } : c
            )
          );
        }}
        onOpenPerson={(personId) =>
          navigate(`/persons/${encodeURIComponent(personId)}`)
        }
        onPrevAsset={
          activeIndex > 0
            ? () => setActiveAssetId(displayedItems[activeIndex - 1].id)
            : undefined
        }
        onNextAsset={
          activeIndex >= 0 && activeIndex < displayedItems.length - 1
            ? () => setActiveAssetId(displayedItems[activeIndex + 1].id)
            : undefined
        }
      />
    </div>
  );
};
