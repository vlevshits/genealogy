import React, { useEffect, useMemo, useState } from 'react';
import { Compass, Layers, MapPin, Minus, Plus, Users } from 'lucide-react';
import { Badge, Button, Card, IconButton } from '../ui';
import { PlaceDto, PlaceMapPinDto, placesApi } from '../../api/client';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PlacesMapViewProps {
  places: PlaceDto[];
  selectedPlaceId?: string | null;
  mapTileUrl?: string;
  onSelectPlace?: (place: PlaceDto) => void;
  className?: string;
}

interface ProjectedPin {
  pin: PlaceMapPinDto;
  svgX: number;
  svgY: number;
}

interface MapCluster {
  id: string;
  svgX: number;
  svgY: number;
  pins: PlaceMapPinDto[];
  totalEvents: number;
  totalPersons: number;
}

interface OsmTileSpec {
  key: string;
  url: string;
  x: number;
  y: number;
  size: number;
}

function lngToTileX(lng: number, z: number): number {
  return ((lng + 180) / 360) * Math.pow(2, z);
}

function latToTileY(lat: number, z: number): number {
  const clampedLat = Math.max(-85.0511, Math.min(85.0511, lat));
  const rad = (clampedLat * Math.PI) / 180;
  const n = Math.pow(2, z);
  return ((1 - Math.log(Math.tan(rad) + 1 / Math.cos(rad)) / Math.PI) / 2) * n;
}

export const PlacesMapView: React.FC<PlacesMapViewProps> = ({
  places,
  selectedPlaceId,
  mapTileUrl = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
  onSelectPlace,
  className,
}) => {
  const { t } = useI18n();
  const [remotePins, setRemotePins] = useState<PlaceMapPinDto[] | null>(null);
  const [zoomOffset, setZoomOffset] = useState<number>(0);
  const [selectedClusterId, setSelectedClusterId] = useState<string | null>(
    null
  );

  useEffect(() => {
    let cancelled = false;
    placesApi
      .getPlacesMap()
      .then((res) => {
        const items = res.items ?? res.pins;
        if (!cancelled && items && items.length > 0) {
          setRemotePins(items);
        }
      })
      .catch(() => {
        // Fallback to geocoded places from props
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const pins: PlaceMapPinDto[] = useMemo(() => {
    const propPins: PlaceMapPinDto[] = places
      .filter(
        (p): p is PlaceDto & { latitude: number; longitude: number } =>
          typeof p.latitude === 'number' &&
          !Number.isNaN(p.latitude) &&
          typeof p.longitude === 'number' &&
          !Number.isNaN(p.longitude)
      )
      .map((p) => ({
        id: p.id,
        gid: p.gid,
        name: p.name,
        full_name: p.full_name,
        place_type: p.place_type,
        latitude: p.latitude,
        longitude: p.longitude,
        events_count: p.events_count,
        persons_count: Math.max(1, Math.ceil(p.events_count * 0.75)),
      }));

    if (remotePins && remotePins.length > 0 && places.length === 0) {
      return remotePins;
    }
    return propPins.length > 0 ? propPins : (remotePins ?? []);
  }, [places, remotePins]);

  const tileCols = 3;
  const tileRows = 2;
  const tileSize = 256;
  const svgW = tileCols * tileSize; // 768
  const svgH = tileRows * tileSize; // 512

  const { clusters, tiles, effectiveZoom } = useMemo(() => {
    if (pins.length === 0) {
      return {
        clusters: [] as MapCluster[],
        tiles: [] as OsmTileSpec[],
        effectiveZoom: 4,
      };
    }

    const lats = pins.map((p) => p.latitude);
    const lngs = pins.map((p) => p.longitude);
    const minLat = Math.min(...lats);
    const maxLat = Math.max(...lats);
    const minLng = Math.min(...lngs);
    const maxLng = Math.max(...lngs);

    let baseZoom = 5;
    const isSingleSpot =
      Math.abs(maxLat - minLat) < 0.02 && Math.abs(maxLng - minLng) < 0.02;

    if (!isSingleSpot) {
      baseZoom = 3;
      for (let z = 9; z >= 3; z -= 1) {
        const dx = Math.abs(lngToTileX(maxLng, z) - lngToTileX(minLng, z));
        const dy = Math.abs(latToTileY(minLat, z) - latToTileY(maxLat, z));
        if (dx <= 1.9 && dy <= 1.2) {
          baseZoom = z;
          break;
        }
      }
    }

    const zoom = Math.max(3, Math.min(9, baseZoom + zoomOffset));
    const n = Math.pow(2, zoom);
    const centerTileX =
      (lngToTileX(minLng, zoom) + lngToTileX(maxLng, zoom)) / 2;
    const centerTileY =
      (latToTileY(maxLat, zoom) + latToTileY(minLat, zoom)) / 2;

    const startTileX = Math.floor(centerTileX - tileCols / 2 + 0.5);
    const startTileY = Math.max(
      0,
      Math.min(n - tileRows, Math.floor(centerTileY - tileRows / 2 + 0.5))
    );

    const tileTemplate =
      mapTileUrl && mapTileUrl.trim().length > 0
        ? mapTileUrl
        : 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';

    const computedTiles: OsmTileSpec[] = [];
    for (let row = 0; row < tileRows; row += 1) {
      for (let col = 0; col < tileCols; col += 1) {
        const rawX = startTileX + col;
        const wrappedX = ((rawX % n) + n) % n;
        const clampedY = Math.max(0, Math.min(n - 1, startTileY + row));
        const url = tileTemplate
          .replace('{z}', String(zoom))
          .replace('{x}', String(wrappedX))
          .replace('{y}', String(clampedY));
        computedTiles.push({
          key: `${zoom}-${wrappedX}-${clampedY}-${col}-${row}`,
          url,
          x: col * tileSize,
          y: row * tileSize,
          size: tileSize,
        });
      }
    }

    const projected: ProjectedPin[] = pins.map((pin) => {
      const tx = lngToTileX(pin.longitude, zoom);
      const ty = latToTileY(pin.latitude, zoom);
      const rawX = Math.round((tx - startTileX) * tileSize);
      const rawY = Math.round((ty - startTileY) * tileSize);
      return {
        pin,
        svgX: Math.max(40, Math.min(svgW - 40, rawX)),
        svgY: Math.max(38, Math.min(svgH - 46, rawY)),
      };
    });

    // Distance-based clustering in pixel space
    const clusterRadius = 64;
    const grouped: MapCluster[] = [];

    for (const item of projected) {
      const existing = grouped.find((c) => {
        const dx = c.svgX - item.svgX;
        const dy = c.svgY - item.svgY;
        return Math.hypot(dx, dy) <= clusterRadius;
      });

      if (existing) {
        const count = existing.pins.length;
        existing.svgX = Math.round(
          (existing.svgX * count + item.svgX) / (count + 1)
        );
        existing.svgY = Math.round(
          (existing.svgY * count + item.svgY) / (count + 1)
        );
        existing.pins.push(item.pin);
        existing.totalEvents += item.pin.events_count;
        existing.totalPersons += item.pin.persons_count;
      } else {
        grouped.push({
          id: `cluster-${item.pin.id}`,
          svgX: item.svgX,
          svgY: item.svgY,
          pins: [item.pin],
          totalEvents: item.pin.events_count,
          totalPersons: item.pin.persons_count,
        });
      }
    }

    return {
      clusters: grouped,
      tiles: computedTiles,
      effectiveZoom: zoom,
    };
  }, [pins, zoomOffset, mapTileUrl, svgW, svgH]);

  const activeCluster = useMemo(() => {
    if (selectedClusterId) {
      const found = clusters.find((c) => c.id === selectedClusterId);
      if (found) return found;
    }
    if (selectedPlaceId) {
      const found = clusters.find((c) =>
        c.pins.some((p) => p.id === selectedPlaceId)
      );
      if (found) return found;
    }
    return clusters[0] ?? null;
  }, [clusters, selectedClusterId, selectedPlaceId]);

  const handleClusterClick = (cluster: MapCluster) => {
    setSelectedClusterId(cluster.id);
    if (cluster.pins.length === 1 && onSelectPlace) {
      const match = places.find((p) => p.id === cluster.pins[0].id);
      if (match) onSelectPlace(match);
    }
  };

  return (
    <Card className={cn('p-5 space-y-4', className)} data-testid="places-map-view">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Compass className="h-4 w-4 text-brand shrink-0" />
          <h2 className="font-serif text-lg font-semibold text-primary">
            {t('places.mapClusterTitle')}
          </h2>
        </div>

        <div className="flex items-center gap-2">
          <Badge variant="brand" size="sm">
            {t('places.mapPlacesCount', { count: pins.length })}
          </Badge>
          <div className="flex items-center gap-1 rounded-xl border border-border bg-surface-muted/60 p-0.5">
            <IconButton
              size="sm"
              variant="ghost"
              label={t('tree.zoomOut')}
              disabled={effectiveZoom <= 3}
              onClick={() => setZoomOffset((prev) => Math.max(-3, prev - 1))}
            >
              <Minus />
            </IconButton>
            <span className="px-2 font-mono text-xs font-medium text-secondary">
              {effectiveZoom}x
            </span>
            <IconButton
              size="sm"
              variant="ghost"
              label={t('tree.zoomIn')}
              disabled={effectiveZoom >= 9}
              onClick={() => setZoomOffset((prev) => Math.min(4, prev + 1))}
            >
              <Plus />
            </IconButton>
          </div>
        </div>
      </div>

      {clusters.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-border bg-surface-muted/40 px-4 py-12 text-center space-y-2">
          <MapPin className="h-7 w-7 text-tertiary" />
          <div className="text-sm font-medium text-secondary">
            {t('places.detail.noCoordinates')}
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="relative overflow-hidden rounded-2xl border border-border bg-surface-muted/60">
            <svg
              viewBox={`0 0 ${svgW} ${svgH}`}
              role="img"
              aria-label={t('places.mapClusterTitle')}
              className="w-full h-72 sm:h-80 block object-cover"
              preserveAspectRatio="xMidYMid slice"
            >
              {/* Base OpenStreetMap Geographical Tiles */}
              <g className="opacity-95">
                {tiles.map((tile) => (
                  <image
                    key={tile.key}
                    href={tile.url}
                    x={tile.x}
                    y={tile.y}
                    width={tile.size}
                    height={tile.size}
                    preserveAspectRatio="none"
                  />
                ))}
              </g>

              <rect
                width={svgW}
                height={svgH}
                className="fill-surface/15 pointer-events-none"
              />

              {/* Clustered & Single Place Pins */}
              {clusters.map((cluster) => {
                const isSelected = activeCluster?.id === cluster.id;
                const isMulti = cluster.pins.length > 1;
                const primaryPin = cluster.pins[0];
                const labelText = isMulti
                  ? `${primaryPin.name} (+${cluster.pins.length - 1})`
                  : primaryPin.name;
                const pillWidth = Math.max(84, labelText.length * 7.2 + 20);
                const radius = isMulti ? 22 : 16;

                return (
                  <g
                    key={cluster.id}
                    role="button"
                    tabIndex={0}
                    aria-label={`${labelText}, ${t('places.mapPlacesCount', { count: cluster.pins.length })}, ${t('places.mapEventsCount', { count: cluster.totalEvents })}`}
                    transform={`translate(${cluster.svgX}, ${cluster.svgY})`}
                    onClick={() => handleClusterClick(cluster)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        handleClusterClick(cluster);
                      }
                    }}
                    className="group cursor-pointer focus:outline-none focus-visible:outline-none"
                    data-testid={`map-cluster-${cluster.id}`}
                  >
                    {isMulti && (
                      <circle
                        r={radius + 6}
                        className="fill-brand/25 stroke-brand/40 stroke-1"
                      />
                    )}
                    <circle
                      r={isSelected ? radius + 2 : radius}
                      className={
                        isSelected
                          ? 'fill-brand stroke-surface stroke-[3] group-focus-visible:stroke-brand group-focus-visible:stroke-[4]'
                          : 'fill-surface stroke-brand stroke-[3] group-focus-visible:stroke-brand group-focus-visible:stroke-[4]'
                      }
                    />
                    <text
                      textAnchor="middle"
                      dy="4.5"
                      className={
                        isSelected
                          ? 'fill-surface font-mono text-[13px] font-bold'
                          : 'fill-brand font-mono text-[13px] font-bold'
                      }
                    >
                      {isMulti ? cluster.pins.length : cluster.totalEvents}
                    </text>

                    <g transform={`translate(0, ${radius + 14})`}>
                      <rect
                        x={-pillWidth / 2}
                        y={-10}
                        width={pillWidth}
                        height={20}
                        rx={6}
                        className="fill-surface/95 stroke-border"
                      />
                      <text
                        textAnchor="middle"
                        dy="4"
                        className="fill-primary font-sans text-[11px] font-semibold"
                      >
                        {labelText}
                      </text>
                    </g>
                  </g>
                );
              })}
            </svg>
          </div>

          {/* Cluster / Place Breakdown Inspector */}
          {activeCluster && (
            <div className="rounded-xl border border-border bg-surface-muted/40 p-3.5 space-y-2.5">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2 text-xs font-semibold text-primary">
                  <Layers className="h-3.5 w-3.5 text-brand" />
                  <span>
                    {t('places.mapPlacesCount', {
                      count: activeCluster.pins.length,
                    })}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="neutral" size="sm">
                    <Users className="h-3 w-3 mr-1 inline" />
                    {t('places.mapPersonsCount', {
                      count: activeCluster.totalPersons,
                    })}
                  </Badge>
                  <Badge variant="brand" size="sm">
                    {t('places.mapEventsCount', {
                      count: activeCluster.totalEvents,
                    })}
                  </Badge>
                </div>
              </div>

              <div className="divide-y divide-border/60">
                {activeCluster.pins.map((pin) => {
                  const fullPlace = places.find((p) => p.id === pin.id);
                  const isCurrent = selectedPlaceId === pin.id;
                  return (
                    <div
                      key={pin.id}
                      className="flex items-center justify-between gap-3 py-2 first:pt-0 last:pb-0"
                    >
                      <div className="min-w-0">
                        <div className="text-xs font-semibold text-primary truncate">
                          {pin.name}
                        </div>
                        <div className="text-[11px] text-secondary truncate">
                          {pin.full_name}
                        </div>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <span className="font-mono text-[11px] text-tertiary">
                          {t('places.mapEventsCount', {
                            count: pin.events_count,
                          })}
                        </span>
                        {fullPlace && onSelectPlace && (
                          <Button
                            size="sm"
                            variant={isCurrent ? 'primary' : 'secondary'}
                            onClick={() => onSelectPlace(fullPlace)}
                          >
                            {t('places.mapInspectPlaceBtn')}
                          </Button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      )}
    </Card>
  );
};
