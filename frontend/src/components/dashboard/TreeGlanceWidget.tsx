import React from 'react';
import {
  AlertTriangle,
  Camera,
  GitBranch,
  MapPin,
  Sparkles,
  Users,
} from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import { TreeGlanceStatsDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface TreeGlanceWidgetProps {
  stats: TreeGlanceStatsDto;
  isAdmin: boolean;
  onOpenQuality?: () => void;
  onOpenPeople?: () => void;
  onOpenPlaces?: () => void;
}

export const TreeGlanceWidget: React.FC<TreeGlanceWidgetProps> = ({
  stats,
  isAdmin,
  onOpenQuality,
  onOpenPeople,
  onOpenPlaces,
}) => {
  const { t } = useI18n();

  const totalPersons =
    stats.total_persons ?? stats.persons_count ?? stats.individuals_count ?? 0;
  const generationsCount =
    stats.generations_count ?? stats.max_generations ?? 0;
  const mediaCount =
    stats.media_count ?? stats.total_photos ?? stats.photos_count ?? 0;
  const placesCount = stats.places_count ?? stats.total_places ?? 0;
  const familiesCount = stats.families_count ?? stats.total_families ?? 0;
  const openWarningsCount =
    stats.open_warnings_count ??
    stats.quality_summary?.total_open_warnings ??
    0;
  const missingBirthCount =
    stats.missing_birth_count ?? stats.missing_birth_dates_count ?? 0;
  const oldestAncestorName =
    typeof stats.oldest_ancestor === 'object' && stats.oldest_ancestor
      ? stats.oldest_ancestor.display_name || ''
      : typeof stats.oldest_ancestor === 'string'
      ? stats.oldest_ancestor
      : stats.oldest_person_name || '';
  const oldestBirthYear =
    stats.earliest_birth_year ??
    (typeof stats.oldest_ancestor === 'object' && stats.oldest_ancestor
      ? stats.oldest_ancestor.birth_year
      : null) ??
    stats.oldest_birth_year ??
    null;

  return (
    <Card
      data-testid="dashboard-tree-glance-widget"
      className="p-5 sm:p-6 space-y-4 flex flex-col justify-between"
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <GitBranch className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('dashboard.glance.title')}
            </h2>
          </div>

          {isAdmin && openWarningsCount > 0 && onOpenQuality && (
            <Button
              size="sm"
              variant="secondary"
              data-testid="dashboard-open-warnings-chip"
              leftIcon={<AlertTriangle className="h-3.5 w-3.5 text-warning" />}
              onClick={onOpenQuality}
            >
              {t('dashboard.glance.warningsChip', {
                count: openWarningsCount,
              })}
            </Button>
          )}
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div
            role={onOpenPeople ? 'button' : undefined}
            tabIndex={onOpenPeople ? 0 : undefined}
            aria-label={
              onOpenPeople
                ? `${t('dashboard.glance.personsLabel')}: ${totalPersons}`
                : undefined
            }
            onClick={onOpenPeople}
            onKeyDown={
              onOpenPeople
                ? (e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      onOpenPeople();
                    }
                  }
                : undefined
            }
            className={
              onOpenPeople
                ? 'rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1 cursor-pointer hover:border-brand/50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus'
                : 'rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1'
            }
          >
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <Users className="h-3.5 w-3.5 text-brand" />
              <span>{t('dashboard.glance.personsLabel')}</span>
            </div>
            <div className="font-mono text-xl font-bold text-primary">
              {totalPersons}
            </div>
          </div>

          <div className="rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <GitBranch className="h-3.5 w-3.5 text-brand" />
              <span>{t('dashboard.glance.generationsLabel')}</span>
            </div>
            <div className="font-mono text-xl font-bold text-primary">
              {generationsCount}
            </div>
          </div>

          <div className="rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <Camera className="h-3.5 w-3.5 text-brand" />
              <span>{t('dashboard.glance.photosLabel')}</span>
            </div>
            <div className="font-mono text-xl font-bold text-primary">
              {mediaCount}
            </div>
          </div>

          <div
            role={onOpenPlaces ? 'button' : undefined}
            tabIndex={onOpenPlaces ? 0 : undefined}
            aria-label={
              onOpenPlaces
                ? `${t('dashboard.glance.placesLabel')}: ${placesCount}`
                : undefined
            }
            onClick={onOpenPlaces}
            onKeyDown={
              onOpenPlaces
                ? (e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      onOpenPlaces();
                    }
                  }
                : undefined
            }
            className={
              onOpenPlaces
                ? 'rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1 cursor-pointer hover:border-brand/50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus'
                : 'rounded-2xl border border-border bg-surface-muted/40 p-3.5 space-y-1'
            }
          >
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <MapPin className="h-3.5 w-3.5 text-brand" />
              <span>{t('dashboard.glance.placesLabel')}</span>
            </div>
            <div className="font-mono text-xl font-bold text-primary">
              {placesCount}
            </div>
          </div>
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-border text-xs text-secondary">
        <span className="inline-flex items-center gap-1.5">
          <Sparkles className="h-3.5 w-3.5 text-brand" />
          {oldestBirthYear
            ? t('dashboard.glance.oldestAncestor', {
                year: oldestBirthYear,
                name: oldestAncestorName,
              })
            : t('dashboard.glance.familiesCount', {
                count: familiesCount,
              })}
        </span>

        {isAdmin && missingBirthCount > 0 && onOpenQuality && (
          <Badge variant="warning" size="sm">
            {t('dashboard.glance.missingBirthsChip', {
              count: missingBirthCount,
            })}
          </Badge>
        )}
      </div>
    </Card>
  );
};
