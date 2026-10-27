import React from 'react';
import { Calendar, Compass, Globe, MapPin } from 'lucide-react';
import {
  Alert,
  Badge,
  Breadcrumbs,
  Card,
  CardBody,
  CardHeader,
  EmptyState,
} from '../ui';
import {
  LifeEventDto,
  PlaceBreadcrumbDto,
  PlaceDto,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface PlaceDetailPanelProps {
  place: PlaceDto | null;
  breadcrumbs?: PlaceBreadcrumbDto[];
  events?: LifeEventDto[];
  mapTileUrl?: string;
  onSelectBreadcrumb?: (placeId: string) => void;
}

export const PlaceDetailPanel: React.FC<PlaceDetailPanelProps> = ({
  place,
  breadcrumbs = [],
  events = [],
  mapTileUrl = '',
  onSelectBreadcrumb,
}) => {
  const { t } = useI18n();

  if (!place) {
    return (
      <Card>
        <CardBody>
          <EmptyState
            icon={<MapPin className="h-6 w-6" />}
            title={t('places.detail.selectPrompt')}
          />
        </CardBody>
      </Card>
    );
  }

  const formatPlaceType = (type: string): string => {
    const key = `places.types.${type}`;
    const translated = t(key);
    return translated === key ? t('places.types.other') : translated;
  };

  const hasCoords =
    place.latitude !== null &&
    place.latitude !== undefined &&
    place.longitude !== null &&
    place.longitude !== undefined;

  const breadcrumbItems =
    breadcrumbs.length > 0
      ? breadcrumbs.map((b) => ({
          id: b.id,
          label: b.name,
          onClick: onSelectBreadcrumb
            ? () => onSelectBreadcrumb(b.id)
            : undefined,
        }))
      : place.full_name.split(',').map((seg, idx) => ({
          id: `seg-${idx}`,
          label: seg.trim(),
        }));

  return (
    <Card>
      <CardHeader className="space-y-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <MapPin className="h-4.5 w-4.5 text-brand" />
            <h3 className="font-sans text-base font-semibold text-primary">
              {place.name}
            </h3>
          </div>
          <div className="flex items-center gap-1.5">
            <Badge variant="brand" size="sm">
              {formatPlaceType(place.place_type)}
            </Badge>
            <Badge variant="neutral" size="sm">
              {place.gid}
            </Badge>
          </div>
        </div>

        <div className="pt-1">
          <Breadcrumbs items={breadcrumbItems} />
        </div>
      </CardHeader>

      <CardBody className="space-y-4">
        {/* Historical & Alternate Names */}
        <div className="space-y-1.5">
          <div className="text-xs font-semibold text-secondary">
            {t('places.detail.altNamesLabel')}
          </div>
          {place.alt_names && place.alt_names.length > 0 ? (
            <div className="flex flex-wrap gap-1.5">
              {place.alt_names.map((alt, idx) => (
                <Badge key={idx} variant="neutral" size="sm">
                  {alt.name}
                  {alt.note ? ` (${alt.note})` : ''}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="text-xs text-tertiary">
              {t('places.detail.noAltNames')}
            </p>
          )}
        </div>

        {/* Coordinates & Self-Hosting Map Tile Rule (12 §4 & §7 DoD #6) */}
        <div className="space-y-1.5">
          <div className="text-xs font-semibold text-secondary">
            {t('places.detail.coordinatesLabel')}
          </div>
          {hasCoords ? (
            <div className="flex items-center gap-2 font-mono text-xs text-primary">
              <Compass className="h-3.5 w-3.5 text-brand" />
              <span>
                {place.latitude?.toFixed(4)}, {place.longitude?.toFixed(4)}
              </span>
            </div>
          ) : (
            <p className="text-xs text-tertiary">
              {t('places.coordsMissing')}
            </p>
          )}

          {!mapTileUrl ? (
            <Alert
              variant="info"
              icon={<Globe className="h-4 w-4 text-brand shrink-0 mt-0.5" />}
            >
              {t('places.detail.mapDisabledNotice')}
            </Alert>
          ) : null}
        </div>

        {/* Chronological Events Recorded Here */}
        <div className="space-y-2 border-t border-border pt-3">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-primary">
              {t('places.detail.eventsHereTitle')}
            </span>
            <Badge variant="neutral" size="sm">
              {events.length || place.events_count}
            </Badge>
          </div>

          {events.length === 0 ? (
            <p className="text-xs text-secondary">
              {t('places.detail.noEventsHere')}
            </p>
          ) : (
            <div className="space-y-2">
              {events.map((ev) => (
                <div
                  key={ev.id}
                  className="rounded-lg border border-border bg-canvas px-3 py-2 text-xs space-y-0.5"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-semibold text-primary capitalize">
                      {ev.event_type}
                    </span>
                    <span className="font-mono text-tertiary">{ev.gid}</span>
                  </div>
                  {(ev.date_text || ev.date?.text) && (
                    <div className="flex items-center gap-1 text-secondary">
                      <Calendar className="h-3 w-3 text-tertiary" />
                      <span>{ev.date_text || ev.date?.text}</span>
                    </div>
                  )}
                  {ev.description && (
                    <div className="text-secondary">{ev.description}</div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </CardBody>
    </Card>
  );
};
