import React, { useState } from 'react';
import {
  Check,
  GitMerge,
  List,
  MapPin,
  Network,
  PenTool,
  Plus,
  Trash2,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  EmptyState,
  SearchInput,
  SegmentedControl,
  Switch,
} from '../ui';
import { PlaceDto } from '../../api/client';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PlacesTreeListCardProps {
  places: PlaceDto[];
  selectedPlaceId?: string | null;
  searchQuery: string;
  onSearchQueryChange: (q: string) => void;
  missingCoordsOnly: boolean;
  onMissingCoordsChange: (checked: boolean) => void;
  canEdit?: boolean;
  initialViewMode?: 'tree' | 'table';
  onSelectPlace: (place: PlaceDto) => void;
  onAddPlace: () => void;
  onEditPlace: (place: PlaceDto) => void;
  onMergePlace: (place: PlaceDto) => void;
  onDeletePlace: (place: PlaceDto) => void;
}

export const PlacesTreeListCard: React.FC<PlacesTreeListCardProps> = ({
  places,
  selectedPlaceId,
  searchQuery,
  onSearchQueryChange,
  missingCoordsOnly,
  onMissingCoordsChange,
  canEdit = true,
  initialViewMode = 'tree',
  onSelectPlace,
  onAddPlace,
  onEditPlace,
  onMergePlace,
  onDeletePlace,
}) => {
  const { t } = useI18n();
  const [viewMode, setViewMode] = useState<'tree' | 'table'>(initialViewMode);

  const formatPlaceType = (type: string): string => {
    const key = `places.types.${type}`;
    const translated = t(key);
    return translated === key ? t('places.types.other') : translated;
  };

  // Compute depth in hierarchy for tree indentation
  const placeMap = new Map<string, PlaceDto>(places.map((p) => [p.id, p]));
  const computeDepth = (place: PlaceDto): number => {
    let depth = 0;
    let currParent = place.parent_id;
    const visited = new Set<string>([place.id]);
    while (currParent && placeMap.has(currParent) && !visited.has(currParent)) {
      visited.add(currParent);
      depth += 1;
      currParent = placeMap.get(currParent)?.parent_id ?? null;
    }
    return depth;
  };

  return (
    <Card>
      <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="space-y-1 min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <MapPin className="h-4.5 w-4.5 text-brand" />
            <h2 className="text-base font-semibold text-primary">
              {t('places.pageTitle')}
            </h2>
            <Badge variant="neutral" size="sm">
              {places.length}
            </Badge>
          </div>
          <p className="text-xs text-secondary">{t('places.pageSubtitle')}</p>
        </div>

        <div className="flex flex-wrap sm:flex-nowrap items-center justify-end gap-2 shrink-0 sm:ml-auto">
          <SegmentedControl
            size="sm"
            ariaLabel={t('places.pageTitle')}
            value={viewMode}
            onChange={(v) => setViewMode(v as 'tree' | 'table')}
            options={[
              {
                value: 'tree',
                label: t('places.viewTree'),
                icon: <Network className="h-3.5 w-3.5" />,
              },
              {
                value: 'table',
                label: t('places.viewTable'),
                icon: <List className="h-3.5 w-3.5" />,
              },
            ]}
          />

          {canEdit && (
            <Button
              size="sm"
              variant="primary"
              leftIcon={<Plus className="h-4 w-4" />}
              onClick={onAddPlace}
              className="shrink-0"
            >
              {t('places.addPlaceBtn')}
            </Button>
          )}
        </div>
      </CardHeader>

      <CardBody className="space-y-4">
        {/* Search & Missing Coordinates Filter Bar */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="flex-1 min-w-0">
            <SearchInput
              value={searchQuery}
              onChange={(e) => onSearchQueryChange(e.target.value)}
              onClear={() => onSearchQueryChange('')}
              placeholder={t('places.searchPlaceholder')}
              aria-label={t('places.searchPlaceholder')}
            />
          </div>
          <div className="shrink-0">
            <Switch
              checked={missingCoordsOnly}
              onCheckedChange={onMissingCoordsChange}
              label={t('places.missingCoordsFilter')}
            />
          </div>
        </div>

        {places.length === 0 ? (
          <EmptyState
            icon={<MapPin className="h-6 w-6" />}
            title={t('places.emptyTitle')}
            description={t('places.emptyDesc')}
            action={
              canEdit ? (
                <Button
                  size="sm"
                  variant="primary"
                  leftIcon={<Plus className="h-4 w-4" />}
                  onClick={onAddPlace}
                >
                  {t('places.addPlaceBtn')}
                </Button>
              ) : undefined
            }
          />
        ) : (
          <div className="divide-y divide-border rounded-xl border border-border bg-canvas overflow-hidden">
            {places.map((pl) => {
              const depth = viewMode === 'tree' ? computeDepth(pl) : 0;
              const isSelected = selectedPlaceId === pl.id;
              const hasCoords =
                pl.latitude !== null &&
                pl.latitude !== undefined &&
                pl.longitude !== null &&
                pl.longitude !== undefined;

              return (
                <div
                  key={pl.id}
                  onClick={() => onSelectPlace(pl)}
                  className={cn(
                    'flex flex-col lg:flex-row lg:items-center justify-between gap-3 px-4 py-3 transition-colors cursor-pointer',
                    isSelected
                      ? 'bg-brand/10'
                      : 'hover:bg-surface-muted/70'
                  )}
                  style={
                    depth > 0
                      ? { paddingLeft: `${Math.min(depth * 1.25 + 1, 5)}rem` }
                      : undefined
                  }
                >
                  <div className="min-w-0 space-y-1 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="font-sans text-sm font-semibold text-primary">
                        {pl.name}
                      </span>
                      <Badge variant="brand" size="sm">
                        {formatPlaceType(pl.place_type)}
                      </Badge>
                      <Badge variant="neutral" size="sm">
                        {pl.gid}
                      </Badge>
                      {pl.alt_names?.map((alt, i) => (
                        <Badge key={i} variant="outline" size="sm">
                          {alt.name}
                        </Badge>
                      ))}
                    </div>

                    {/* Full hierarchy disambiguation line (12 §3.1 & §7 DoD #2) */}
                    <div className="text-xs text-secondary truncate">
                      {pl.full_name}
                    </div>
                  </div>

                  <div
                    className="flex flex-wrap items-center gap-2 shrink-0"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <Badge variant="neutral" size="sm">
                      {t('places.colEvents')}: {pl.events_count}
                    </Badge>
                    <Badge
                      variant={hasCoords ? 'success' : 'warning'}
                      size="sm"
                      icon={
                        hasCoords ? <Check className="h-3 w-3" /> : undefined
                      }
                    >
                      {hasCoords
                        ? t('places.coordsPresent')
                        : t('places.coordsMissing')}
                    </Badge>

                    {canEdit && (
                      <>
                        <Button
                          size="sm"
                          variant="secondary"
                          leftIcon={<PenTool className="h-3 w-3" />}
                          onClick={() => onEditPlace(pl)}
                        >
                          {t('places.editPlaceBtn')}
                        </Button>
                        <Button
                          size="sm"
                          variant="secondary"
                          leftIcon={<GitMerge className="h-3 w-3" />}
                          onClick={() => onMergePlace(pl)}
                        >
                          {t('places.mergePlaceBtn')}
                        </Button>
                        {pl.events_count === 0 && pl.children_count === 0 && (
                          <Button
                            size="sm"
                            variant="ghost"
                            leftIcon={
                              <Trash2 className="h-3.5 w-3.5 text-danger" />
                            }
                            onClick={() => onDeletePlace(pl)}
                          >
                            {t('places.deletePlaceBtn')}
                          </Button>
                        )}
                      </>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardBody>
    </Card>
  );
};
