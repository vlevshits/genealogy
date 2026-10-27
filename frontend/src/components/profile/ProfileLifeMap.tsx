import React from 'react';
import { Compass, MapPin, Navigation } from 'lucide-react';
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

export const ProfileLifeMap: React.FC<ProfileLifeMapProps> = ({
  events,
  highlightedEventId,
  onSelectEvent,
}) => {
  const { t } = useI18n();

  const geocodedRaw = events.filter(
    (ev): ev is ProfileLifeEventDto & { latitude: number; longitude: number } =>
      typeof ev.latitude === 'number' &&
      !Number.isNaN(ev.latitude) &&
      typeof ev.longitude === 'number' &&
      !Number.isNaN(ev.longitude)
  );

  // Project lat/lng into a normalized 480x260 SVG canvas with padding
  const svgW = 480;
  const svgH = 240;
  const padX = 56;
  const padY = 44;

  const milestones: GeocodedMilestone[] = React.useMemo(() => {
    if (geocodedRaw.length === 0) return [];
    const lats = geocodedRaw.map((g) => g.latitude);
    const lngs = geocodedRaw.map((g) => g.longitude);
    const minLat = Math.min(...lats);
    const maxLat = Math.max(...lats);
    const minLng = Math.min(...lngs);
    const maxLng = Math.max(...lngs);

    const latSpan = Math.max(0.5, maxLat - minLat);
    const lngSpan = Math.max(0.5, maxLng - minLng);

    return geocodedRaw.map((ev, idx) => {
      const normX =
        geocodedRaw.length === 1 ? 0.5 : (ev.longitude - minLng) / lngSpan;
      // Higher latitude -> smaller Y (north is up)
      const normY =
        geocodedRaw.length === 1 ? 0.5 : 1 - (ev.latitude - minLat) / latSpan;

      const svgX = Math.round(padX + normX * (svgW - padX * 2));
      const svgY = Math.round(padY + normY * (svgH - padY * 2));
      const year = ev.date_sort ? Math.floor(ev.date_sort / 10000) : null;

      return {
        index: idx + 1,
        event: ev,
        lat: ev.latitude,
        lng: ev.longitude,
        svgX,
        svgY,
        year,
      };
    });
  }, [geocodedRaw]);

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
          {/* Interactive SVG Migration Canvas */}
          <div className="relative overflow-hidden rounded-2xl border border-border bg-surface-muted/40">
            <svg
              viewBox={`0 0 ${svgW} ${svgH}`}
              role="img"
              aria-label={t('profile.map.svgAriaLabel')}
              className="w-full h-52 block"
            >
              <defs>
                <pattern
                  id="life-map-grid"
                  width="32"
                  height="32"
                  patternUnits="userSpaceOnUse"
                >
                  <path
                    d="M 32 0 L 0 0 0 32"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="0.5"
                    className="text-border/60"
                  />
                </pattern>
                <marker
                  id="migration-arrow"
                  viewBox="0 0 10 10"
                  refX="8"
                  refY="5"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path
                    d="M 0 1 L 8 5 L 0 9 z"
                    className="fill-brand"
                  />
                </marker>
              </defs>

              <rect width={svgW} height={svgH} fill="url(#life-map-grid)" />

              {/* Migration Path Segments */}
              {milestones.slice(0, -1).map((from, idx) => {
                const to = milestones[idx + 1];
                return (
                  <line
                    key={`${from.event.id}-${to.event.id}`}
                    x1={from.svgX}
                    y1={from.svgY}
                    x2={to.svgX}
                    y2={to.svgY}
                    strokeWidth={2.25}
                    strokeDasharray="5 3"
                    markerEnd="url(#migration-arrow)"
                    className="stroke-brand/80"
                  />
                );
              })}

              {/* Numbered Geocoded Event Pins */}
              {milestones.map((m) => {
                const isSelected = highlightedEventId === m.event.id;
                const shortPlace = (m.event.place_full_name || '').split(',')[0];
                return (
                  <g
                    key={m.event.id}
                    transform={`translate(${m.svgX}, ${m.svgY})`}
                    onClick={() => onSelectEvent?.(m.event.id)}
                    className="cursor-pointer"
                  >
                    <circle
                      r={isSelected ? 15 : 12}
                      className={
                        isSelected
                          ? 'fill-brand stroke-surface stroke-2'
                          : 'fill-surface stroke-brand stroke-2'
                      }
                    />
                    <text
                      textAnchor="middle"
                      dy="4"
                      className={
                        isSelected
                          ? 'fill-surface font-mono text-[11px] font-bold'
                          : 'fill-brand font-mono text-[11px] font-bold'
                      }
                    >
                      {m.index}
                    </text>
                    <text
                      textAnchor="middle"
                      dy="26"
                      className="fill-primary font-sans text-[10px] font-medium"
                    >
                      {shortPlace}
                      {m.year ? ` (${m.year})` : ''}
                    </text>
                  </g>
                );
              })}
            </svg>
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
