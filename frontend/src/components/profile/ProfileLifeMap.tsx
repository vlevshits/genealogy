import React from 'react';
import { Compass, ExternalLink, MapPin, Navigation } from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import { ProfileLifeEventDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface ProfileLifeMapProps {
  events: ProfileLifeEventDto[];
  highlightedEventId?: string | null;
  onSelectEvent?: (eventId: string) => void;
}

interface GeocodedMilestone {
  index: number;
  event: ProfileLifeEventDto;
  lat: number;
  lng: number;
  svgX: number;
  svgY: number;
  year: number | null;
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

export const ProfileLifeMap: React.FC<ProfileLifeMapProps> = ({
  events,
  highlightedEventId,
  onSelectEvent,
}) => {
  const { t } = useI18n();

  const geocodedRaw = events.filter(
    (
      ev
    ): ev is ProfileLifeEventDto & { latitude: number; longitude: number } =>
      typeof ev.latitude === 'number' &&
      !Number.isNaN(ev.latitude) &&
      typeof ev.longitude === 'number' &&
      !Number.isNaN(ev.longitude)
  );

  // 3x2 grid of 256x256 OpenStreetMap tiles -> 768x512 Web Mercator viewport
  const tileCols = 3;
  const tileRows = 2;
  const tileSize = 256;
  const svgW = tileCols * tileSize; // 768
  const svgH = tileRows * tileSize; // 512

  const { milestones, tiles, osmViewUrl } = React.useMemo(() => {
    if (geocodedRaw.length === 0) {
      return {
        milestones: [] as GeocodedMilestone[],
        tiles: [] as OsmTileSpec[],
        osmViewUrl: 'https://www.openstreetmap.org',
      };
    }

    const lats = geocodedRaw.map((g) => g.latitude);
    const lngs = geocodedRaw.map((g) => g.longitude);
    const minLat = Math.min(...lats);
    const maxLat = Math.max(...lats);
    const minLng = Math.min(...lngs);
    const maxLng = Math.max(...lngs);

    // Pick highest zoom z in [3..10] where bounding box fits comfortably within ~1.8 x 1.1 tiles
    let zoom = 8;
    const isSingleSpot =
      Math.abs(maxLat - minLat) < 0.01 && Math.abs(maxLng - minLng) < 0.01;

    if (!isSingleSpot) {
      zoom = 3;
      for (let z = 10; z >= 3; z -= 1) {
        const dx = Math.abs(lngToTileX(maxLng, z) - lngToTileX(minLng, z));
        const dy = Math.abs(latToTileY(minLat, z) - latToTileY(maxLat, z));
        if (dx <= 1.8 && dy <= 1.15) {
          zoom = z;
          break;
        }
      }
    }

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

    const computedTiles: OsmTileSpec[] = [];
    for (let row = 0; row < tileRows; row += 1) {
      for (let col = 0; col < tileCols; col += 1) {
        const rawX = startTileX + col;
        const wrappedX = ((rawX % n) + n) % n;
        const clampedY = Math.max(0, Math.min(n - 1, startTileY + row));
        computedTiles.push({
          key: `${zoom}-${wrappedX}-${clampedY}-${col}-${row}`,
          url: `https://tile.openstreetmap.org/${zoom}/${wrappedX}/${clampedY}.png`,
          x: col * tileSize,
          y: row * tileSize,
          size: tileSize,
        });
      }
    }

    const computedMilestones: GeocodedMilestone[] = geocodedRaw.map(
      (ev, idx) => {
        const tx = lngToTileX(ev.longitude, zoom);
        const ty = latToTileY(ev.latitude, zoom);
        const svgX = Math.round((tx - startTileX) * tileSize);
        const svgY = Math.round((ty - startTileY) * tileSize);
        const year = ev.date_sort ? Math.floor(ev.date_sort / 10000) : null;

        return {
          index: idx + 1,
          event: ev,
          lat: ev.latitude,
          lng: ev.longitude,
          svgX: Math.max(36, Math.min(svgW - 36, svgX)),
          svgY: Math.max(32, Math.min(svgH - 44, svgY)),
          year,
        };
      }
    );

    const centerLat = ((minLat + maxLat) / 2).toFixed(4);
    const centerLng = ((minLng + maxLng) / 2).toFixed(4);
    const computedOsmUrl = `https://www.openstreetmap.org/#map=${zoom}/${centerLat}/${centerLng}`;

    return {
      milestones: computedMilestones,
      tiles: computedTiles,
      osmViewUrl: computedOsmUrl,
    };
  }, [geocodedRaw, svgH, svgW]);

  return (
    <Card className="p-5 space-y-4">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Compass className="h-4 w-4 text-brand shrink-0" />
          <h3 className="font-sans text-sm font-semibold text-primary">
            {t('profile.map.title')}
          </h3>
        </div>
        <Badge variant="neutral" size="sm">
          {t('profile.map.stopsCount', { count: milestones.length })}
        </Badge>
      </div>

      {milestones.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-border bg-surface-muted/40 px-4 py-8 text-center space-y-2">
          <MapPin className="h-6 w-6 text-tertiary" />
          <div className="text-xs font-medium text-secondary">
            {t('profile.map.emptyTitle')}
          </div>
          <p className="text-[11px] text-tertiary max-w-xs">
            {t('profile.map.emptyDesc')}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {/* Geographical OpenStreetMap Tile Canvas + SVG Migration Overlay */}
          <div className="relative overflow-hidden rounded-2xl border border-border bg-surface-muted/60">
            <svg
              viewBox={`0 0 ${svgW} ${svgH}`}
              role="img"
              aria-label={t('profile.map.svgAriaLabel')}
              className="w-full h-60 sm:h-64 block object-cover"
              preserveAspectRatio="xMidYMid slice"
            >
              <defs>
                <marker
                  id="migration-arrow"
                  viewBox="0 0 10 10"
                  refX="8"
                  refY="5"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 1 L 8 5 L 0 9 z" className="fill-brand" />
                </marker>
              </defs>

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

              {/* Subtle contrast scrim so pins and labels remain legible over map labels */}
              <rect
                width={svgW}
                height={svgH}
                className="fill-surface/15 pointer-events-none"
              />

              {/* Migration Path Segments */}
              {milestones.slice(0, -1).map((from, idx) => {
                const to = milestones[idx + 1];
                return (
                  <g key={`${from.event.id}-${to.event.id}`}>
                    <line
                      x1={from.svgX}
                      y1={from.svgY}
                      x2={to.svgX}
                      y2={to.svgY}
                      strokeWidth={5}
                      className="stroke-surface/80"
                    />
                    <line
                      x1={from.svgX}
                      y1={from.svgY}
                      x2={to.svgX}
                      y2={to.svgY}
                      strokeWidth={2.75}
                      strokeDasharray="7 4"
                      markerEnd="url(#migration-arrow)"
                      className="stroke-brand"
                    />
                  </g>
                );
              })}

              {/* Numbered Geocoded Event Pins */}
              {milestones.map((m) => {
                const isSelected = highlightedEventId === m.event.id;
                const shortPlace = (m.event.place_full_name || '').split(
                  ','
                )[0];
                const labelText = `${shortPlace}${m.year ? ` (${m.year})` : ''}`;
                const pillWidth = Math.max(76, labelText.length * 7.2 + 16);

                return (
                  <g
                    key={m.event.id}
                    transform={`translate(${m.svgX}, ${m.svgY})`}
                    onClick={() => onSelectEvent?.(m.event.id)}
                    className="cursor-pointer"
                  >
                    <circle
                      r={isSelected ? 18 : 15}
                      className={
                        isSelected
                          ? 'fill-brand stroke-surface stroke-[3]'
                          : 'fill-surface stroke-brand stroke-[3]'
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
                      {m.index}
                    </text>

                    {/* High-contrast pill label beneath pin */}
                    <g transform="translate(0, 22)">
                      <rect
                        x={-pillWidth / 2}
                        y={-9}
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

            {/* OpenStreetMap Attribution & External Link Overlay */}
            <div className="flex items-center justify-between gap-2 border-t border-border/70 bg-surface/90 px-3 py-1.5 text-[11px] text-secondary backdrop-blur-xs">
              <span>{t('profile.map.osmAttribution')}</span>
              <a
                href={osmViewUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 font-medium text-brand hover:underline"
              >
                <span>{t('profile.map.openInOsm')}</span>
                <ExternalLink className="h-3 w-3" />
              </a>
            </div>
          </div>

          {/* Interactive Stop List */}
          <div className="flex flex-wrap items-center gap-1.5">
            {milestones.map((m) => {
              const isSelected = highlightedEventId === m.event.id;
              return (
                <Button
                  key={m.event.id}
                  size="sm"
                  variant={isSelected ? 'primary' : 'secondary'}
                  leftIcon={<Navigation className="h-3 w-3" />}
                  onClick={() => onSelectEvent?.(m.event.id)}
                  className="text-xs"
                >
                  {m.index}. {(m.event.place_full_name || '').split(',')[0]}
                  {m.year ? ` (${m.year})` : ''}
                </Button>
              );
            })}
          </div>
        </div>
      )}
    </Card>
  );
};
