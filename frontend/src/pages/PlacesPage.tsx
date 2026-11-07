import React, { useEffect, useState } from 'react';
import { ListTree, Map as MapIcon } from 'lucide-react';
import { SegmentedControl, useToast } from '../components/ui';
import {
  PlaceDetailPanel,
  PlaceFormDialog,
  PlaceMergeDialog,
  PlacesMapView,
  PlacesTreeListCard,
} from '../components/places';
import { PlacesQualityBanner } from '../components/quality';
import {
  deletePlace,
  fetchPlaceDetail,
  fetchPlaces,
  LifeEventDto,
  PlaceBreadcrumbDto,
  PlaceDto,
  qualityApi,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';
import { matchesCrossScript } from '../lib/utils';

export interface PlacesPageProps {
  initialPlaces?: PlaceDto[];
  initialMapTileUrl?: string;
  initialView?: 'list' | 'map';
  initialQualityIssuesCount?: number;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export const DEFAULT_PLACES: PlaceDto[] = [
  {
    id: 'pl-empire',
    gid: 'P0001',
    name: 'Российская империя',
    place_type: 'country',
    parent_id: null,
    full_name: 'Российская империя',
    latitude: null,
    longitude: null,
    alt_names: [{ name: 'Russian Empire', lang: 'en' }],
    events_count: 0,
    children_count: 2,
    version: 1,
  },
  {
    id: 'pl-tver-gov',
    gid: 'P0002',
    name: 'Тверская губерния',
    place_type: 'region',
    parent_id: 'pl-empire',
    full_name: 'Тверская губерния, Российская империя',
    latitude: 57.0,
    longitude: 35.5,
    alt_names: [{ name: 'Tver Governorate', lang: 'en' }],
    events_count: 2,
    children_count: 1,
    version: 1,
  },
  {
    id: 'pl-tver-city',
    gid: 'P0003',
    name: 'Тверь',
    place_type: 'city',
    parent_id: 'pl-tver-gov',
    full_name: 'Тверь, Тверская губерния, Российская империя',
    latitude: 56.8587,
    longitude: 35.9176,
    alt_names: [
      { name: 'Калинин', lang: 'ru', note: '1931–1990' },
      { name: 'Tver', lang: 'en' },
    ],
    events_count: 6,
    children_count: 0,
    version: 1,
  },
  {
    id: 'pl-samara-gov',
    gid: 'P0004',
    name: 'Самарская губерния',
    place_type: 'region',
    parent_id: 'pl-empire',
    full_name: 'Самарская губерния, Российская империя',
    latitude: 53.2,
    longitude: 50.15,
    alt_names: [{ name: 'Samara Governorate', lang: 'en' }],
    events_count: 1,
    children_count: 1,
    version: 1,
  },
  {
    id: 'pl-pokrovka',
    gid: 'P0005',
    name: 'Покровка',
    place_type: 'village',
    parent_id: 'pl-samara-gov',
    full_name: 'Покровка, Новоузенский уезд, Самарская губерния, Российская империя',
    latitude: null,
    longitude: null,
    alt_names: [{ name: 'Pokrovka', lang: 'en' }],
    events_count: 4,
    children_count: 0,
    version: 1,
  },
];

export const PlacesPage: React.FC<PlacesPageProps> = ({
  initialPlaces,
  initialMapTileUrl = '',
  initialView,
  initialQualityIssuesCount,
  userRole,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isAdmin = effectiveRole === 'admin';
  const canEdit = effectiveRole === 'admin' || effectiveRole === 'editor';

  const resolvedInitialView: 'list' | 'map' =
    initialView ??
    (typeof window !== 'undefined' &&
    new URLSearchParams(window.location.search).get('view') === 'map'
      ? 'map'
      : 'list');
  const [viewMode, setViewMode] = useState<'list' | 'map'>(resolvedInitialView);

  const [places, setPlaces] = useState<PlaceDto[]>(
    initialPlaces ?? DEFAULT_PLACES
  );
  const [mapTileUrl, setMapTileUrl] = useState<string>(initialMapTileUrl);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [missingCoordsOnly, setMissingCoordsOnly] = useState<boolean>(false);

  const [selectedPlace, setSelectedPlace] = useState<PlaceDto | null>(
    initialPlaces ? (initialPlaces[2] ?? initialPlaces[0] ?? null) : null
  );
  const [detailBreadcrumbs, setDetailBreadcrumbs] = useState<
    PlaceBreadcrumbDto[]
  >([]);
  const [detailEvents, setDetailEvents] = useState<LifeEventDto[]>([]);

  const [formOpen, setFormOpen] = useState<boolean>(false);
  const [editingPlace, setEditingPlace] = useState<PlaceDto | null>(null);
  const [mergingPlace, setMergingPlace] = useState<PlaceDto | null>(null);
  const [qualityIssuesCount, setQualityIssuesCount] = useState<number>(
    initialQualityIssuesCount ?? 3
  );
  const [highConfCount, setHighConfCount] = useState<number>(2);

  const navigateTo = (nextPath: string) => {
    if (onNavigate) {
      onNavigate(nextPath);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', nextPath);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  useEffect(() => {
    if (!isAdmin || initialQualityIssuesCount !== undefined) return;
    let cancelled = false;
    qualityApi
      .getPlaceHierarchyAnalysis()
      .then((res) => {
        if (!cancelled && res) {
          setQualityIssuesCount(res.counts?.total ?? res.items.length);
          setHighConfCount(
            res.counts?.high_confidence ??
              res.items.filter((i) => i.confidence_tier === 'high').length
          );
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [isAdmin, initialQualityIssuesCount]);

  const loadPlaces = (q?: string, missingCoords?: boolean) => {
    let cancelled = false;
    fetchPlaces({
      q: q?.trim() || undefined,
      missing_coords: missingCoords || undefined,
    })
      .then((res) => {
        if (cancelled) return;
        setPlaces(res.items);
        if (res.map_tile_url !== undefined) {
          setMapTileUrl(res.map_tile_url);
        }
        setSelectedPlace((prev) => {
          if (
            res.items.length > 0 &&
            (!prev || !res.items.some((p) => p.id === prev.id))
          ) {
            return res.items[0];
          }
          return prev;
        });
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  };

  useEffect(() => {
    if (initialPlaces) return;
    return loadPlaces(searchQuery, missingCoordsOnly);
  }, [initialPlaces, searchQuery, missingCoordsOnly]);

  useEffect(() => {
    if (
      !selectedPlace ||
      initialPlaces ||
      selectedPlace.id.startsWith('pl-')
    ) {
      return;
    }
    let cancelled = false;
    fetchPlaceDetail(selectedPlace.id)
      .then((res) => {
        if (!cancelled) {
          setDetailBreadcrumbs(res.breadcrumbs);
          setDetailEvents(res.events);
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [selectedPlace, initialPlaces]);

  // Client-side filter fallback (also supports cross-script & alt-name search)
  const filteredPlaces = places.filter((pl) => {
    if (missingCoordsOnly) {
      const hasCoords =
        pl.latitude !== null &&
        pl.latitude !== undefined &&
        pl.longitude !== null &&
        pl.longitude !== undefined;
      if (hasCoords) return false;
    }
    if (!searchQuery.trim()) return true;
    const q = searchQuery.trim();
    if (matchesCrossScript(pl.name, q)) return true;
    if (matchesCrossScript(pl.full_name, q)) return true;
    if (
      pl.alt_names?.some((alt) => matchesCrossScript(alt.name, q))
    ) {
      return true;
    }
    return false;
  });

  const handleDeletePlace = async (pl: PlaceDto) => {
    try {
      await deletePlace(pl.id);
      setPlaces((prev) => prev.filter((item) => item.id !== pl.id));
      if (selectedPlace?.id === pl.id) {
        setSelectedPlace(null);
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  return (
    <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="font-serif text-2xl font-semibold text-primary">
            {t('places.title')}
          </h1>
          <p className="text-xs text-secondary mt-0.5">
            {t('places.subtitle')}
          </p>
        </div>

        <SegmentedControl<'list' | 'map'>
          size="sm"
          aria-label={t('places.viewModeAria')}
          value={viewMode}
          onChange={setViewMode}
          options={[
            {
              value: 'list',
              label: t('places.viewList'),
              icon: <ListTree className="h-3.5 w-3.5" />,
            },
            {
              value: 'map',
              label: t('places.viewMap'),
              icon: <MapIcon className="h-3.5 w-3.5" />,
            },
          ]}
        />
      </div>

      {isAdmin && qualityIssuesCount > 0 && (
        <PlacesQualityBanner
          issueCount={qualityIssuesCount}
          highConfidenceCount={highConfCount}
          onReviewFixes={() => navigateTo('/quality?tab=places_hierarchy')}
        />
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start">
        <div className="lg:col-span-2 min-w-0 space-y-6">
          {viewMode === 'map' ? (
            <PlacesMapView
              places={filteredPlaces}
              selectedPlaceId={selectedPlace?.id ?? null}
              mapTileUrl={mapTileUrl}
              onSelectPlace={(pl) => setSelectedPlace(pl)}
            />
          ) : (
            <PlacesTreeListCard
              places={filteredPlaces}
              selectedPlaceId={selectedPlace?.id ?? null}
              searchQuery={searchQuery}
              onSearchQueryChange={setSearchQuery}
              missingCoordsOnly={missingCoordsOnly}
              onMissingCoordsChange={setMissingCoordsOnly}
              canEdit={canEdit}
              onSelectPlace={(pl) => setSelectedPlace(pl)}
              onAddPlace={() => {
                setEditingPlace(null);
                setFormOpen(true);
              }}
              onEditPlace={(pl) => {
                setEditingPlace(pl);
                setFormOpen(true);
              }}
              onMergePlace={(pl) => setMergingPlace(pl)}
              onDeletePlace={handleDeletePlace}
            />
          )}
        </div>

        <div className="lg:col-span-1 min-w-0">
          <PlaceDetailPanel
            place={selectedPlace}
            breadcrumbs={detailBreadcrumbs}
            events={detailEvents}
            mapTileUrl={mapTileUrl}
            onSelectBreadcrumb={(id) => {
              const found = places.find((p) => p.id === id);
              if (found) setSelectedPlace(found);
            }}
          />
        </div>
      </div>

      <PlaceFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        initialPlace={editingPlace}
        availableParents={places}
        onPlaceSaved={(saved) => {
          setPlaces((prev) => {
            const exists = prev.some((p) => p.id === saved.id);
            return exists
              ? prev.map((p) => (p.id === saved.id ? saved : p))
              : [...prev, saved];
          });
          setSelectedPlace(saved);
          loadPlaces(searchQuery, missingCoordsOnly);
        }}
      />

      <PlaceMergeDialog
        open={Boolean(mergingPlace)}
        onOpenChange={(open) => {
          if (!open) setMergingPlace(null);
        }}
        sourcePlace={mergingPlace}
        availableTargets={places}
        onMerged={(targetPlace, mergedSourceId) => {
          setPlaces((prev) =>
            prev
              .filter((p) => p.id !== mergedSourceId)
              .map((p) => (p.id === targetPlace.id ? targetPlace : p))
          );
          setSelectedPlace(targetPlace);
          loadPlaces(searchQuery, missingCoordsOnly);
        }}
      />
    </div>
  );
};

export default PlacesPage;
