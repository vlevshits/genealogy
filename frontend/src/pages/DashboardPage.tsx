import React, { useCallback, useEffect, useState } from 'react';
import { Home, UserPlus } from 'lucide-react';
import {
  Badge,
  BrandLogoIcon,
  Button,
  useToast,
} from '../components/ui';
import {
  AnniversariesWidget,
  LatestPhotosWidget,
  RecentActivityWidget,
  TreeGlanceWidget,
} from '../components/dashboard';
import { PublicGuestBanner } from '../components/tree/PublicGuestBanner';
import { QuickAddRelativeDialog } from '../components/person/QuickAddRelativeDialog';
import { MediaLightboxStudio } from '../components/media/MediaLightboxStudio';
import {
  dashboardApi,
  DashboardPayloadDto,
  MediaAssetSummaryDto,
  undoTransaction,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { useI18n } from '../i18n';

export interface DashboardPageProps {
  initialDashboard?: DashboardPayloadDto;
  instanceTitle?: string;
  resolvedHomePersonId?: string | null;
  accessTier?: 'public_anonymous' | 'viewer' | 'editor' | 'admin';
  onNavigate?: (path: string) => void;
  onSignIn?: () => void;
}

export function buildDefaultDashboardPayload(): DashboardPayloadDto {
  return {
    resolved_home_person_id: 'node-deceased-1',
    anniversaries: [
      {
        id: 'ann-1',
        type: 'birthday',
        person_id: 'node-living-1',
        display_name: 'Elena Alexandrovna Levshits',
        is_living: true,
        date_month_day: '06-14',
        date_label: 'Jun 14',
        years_count: 32,
        birth_year: 1994,
        days_until: 0,
      },
      {
        id: 'ann-2',
        type: 'memorial',
        person_id: 'node-deceased-1',
        display_name: 'Alexander Mikhailovich Levshits',
        is_living: false,
        date_month_day: '03-19',
        date_label: 'Mar 19',
        years_count: 115,
        birth_year: 1912,
        death_year: 1989,
        days_until: 5,
      },
      {
        id: 'ann-3',
        type: 'wedding',
        person_id: 'node-deceased-1',
        partner_person_id: 'node-deceased-2',
        display_name: 'Alexander & Sofia Levshits',
        is_living: false,
        date_month_day: '11-07',
        date_label: 'Nov 7',
        years_count: 90,
        days_until: 10,
      },
    ],
    recent_activity: [
      {
        tx_id: 'tx-dash-1',
        tx_label: 'person.add_parent',
        actor_name: 'Vera Levshits',
        created_at: '2026-10-27T14:32:00Z',
        can_undo: true,
        entries: [
          {
            id: 'chg-dash-1',
            tx_id: 'tx-dash-1',
            tx_label: 'person.add_parent',
            entity_type: 'person',
            entity_id: 'node-deceased-1',
            entity_label: 'Pyotr Fyodorovich Shcheglov',
            op: 'create',
            created_at: '2026-10-27T14:32:00Z',
          },
        ],
      },
      {
        tx_id: 'tx-dash-2',
        tx_label: 'person.update_profile',
        actor_name: 'Lev Levshits',
        created_at: '2026-10-27T11:15:00Z',
        can_undo: true,
        entries: [
          {
            id: 'chg-dash-2',
            tx_id: 'tx-dash-2',
            tx_label: 'person.update_profile',
            entity_type: 'person',
            entity_id: 'node-deceased-2',
            entity_label: 'Sofia Yakovlevna Levshits',
            op: 'update',
            created_at: '2026-10-27T11:15:00Z',
          },
        ],
      },
    ],
    stats: {
      total_persons: 142,
      total_families: 48,
      total_photos: 64,
      total_places: 29,
      max_generations: 7,
      oldest_birth_year: 1788,
      oldest_person_name: 'Fyodor Shcheglov',
      open_warnings_count: 4,
      missing_birth_dates_count: 9,
    },
    latest_photos: [
      {
        id: 'med-1',
        gid: 'M0001',
        title: 'Leningrad Family Portrait, 1954',
        mime_type: 'image/svg+xml',
        file_size_bytes: 245000,
        date_taken: '1954-05-12',
        place_full_name: 'Leningrad',
        is_private: false,
        only_living_persons: false,
        linked_person_ids: ['node-deceased-1'],
        file_url:
          'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300"><rect width="400" height="300" fill="%23e5dfd5"/><circle cx="150" cy="120" r="36" fill="%238c7a6b"/><circle cx="250" cy="125" r="34" fill="%238c7a6b"/></svg>',
        face_tags: [
          {
            id: 'ft-1',
            person_id: 'node-deceased-1',
            person_name: 'Alexander Levshits',
            is_living: false,
            x: 0.28,
            y: 0.25,
            width: 0.18,
            height: 0.24,
            is_primary_avatar: true,
          },
        ],
        links: [],
        version: 1,
      },
      {
        id: 'med-2',
        gid: 'M0002',
        title: 'Wedding in Vitebsk, 1936',
        mime_type: 'image/svg+xml',
        file_size_bytes: 198000,
        date_taken: '1936-08-19',
        place_full_name: 'Vitebsk',
        is_private: false,
        only_living_persons: false,
        linked_person_ids: [],
        file_url:
          'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300"><rect width="400" height="300" fill="%23dfd8cc"/><circle cx="200" cy="130" r="40" fill="%237a6a58"/></svg>',
        face_tags: [],
        links: [],
        version: 1,
      },
    ],
  };
}

export const DashboardPage: React.FC<DashboardPageProps> = ({
  initialDashboard,
  instanceTitle,
  resolvedHomePersonId: propHomePersonId,
  accessTier: propAccessTier,
  onNavigate,
  onSignIn,
}) => {
  const { t } = useI18n();
  const { user, isAuthenticated } = useAuth();
  const { addToast } = useToast();

  const effectiveTier =
    propAccessTier ??
    (user?.role
      ? user.role
      : isAuthenticated
      ? 'viewer'
      : 'public_anonymous');
  const isAdmin = effectiveTier === 'admin';
  const canEdit = effectiveTier === 'admin' || effectiveTier === 'editor';

  const [scope, setScope] = useState<'all' | 'close'>('all');
  const [dashboard, setDashboard] = useState<DashboardPayloadDto>(
    () => initialDashboard ?? buildDefaultDashboardPayload()
  );
  const [quickAddOpen, setQuickAddOpen] = useState<boolean>(false);
  const [previewPhoto, setPreviewPhoto] =
    useState<MediaAssetSummaryDto | null>(null);

  const navigateTo = useCallback(
    (nextPath: string) => {
      if (onNavigate) {
        onNavigate(nextPath);
      } else if (typeof window !== 'undefined') {
        window.history.pushState({}, '', nextPath);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }
    },
    [onNavigate]
  );

  const loadDashboard = useCallback(
    (targetScope: 'all' | 'close') => {
      if (initialDashboard) return;
      dashboardApi
        .getDashboard(targetScope)
        .then((res) => {
          setDashboard((prev) => ({
            anniversaries:
              res.anniversaries ??
              res.upcoming_anniversaries ??
              prev.anniversaries,
            recent_activity: res.recent_activity ?? prev.recent_activity,
            stats: res.stats ?? res.tree_stats ?? prev.stats,
            latest_photos: res.latest_photos ?? prev.latest_photos,
            resolved_home_person_id:
              res.resolved_home_person_id ?? prev.resolved_home_person_id,
          }));
        })
        .catch(() => {});
    },
    [initialDashboard]
  );

  useEffect(() => {
    loadDashboard(scope);
  }, [loadDashboard, scope]);

  const handleUndoTx = async (txId: string) => {
    try {
      await undoTransaction(txId).catch(() => ({
        tx_id: txId,
        reverted_count: 1,
        status: 'undone',
      }));
      setDashboard((prev) => ({
        ...prev,
        recent_activity: prev.recent_activity.map((tx) =>
          tx.tx_id === txId ? { ...tx, can_undo: false } : tx
        ),
      }));
      addToast({
        type: 'success',
        title: t('history.undo_confirmed_toast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const homePersonId =
    propHomePersonId ??
    dashboard.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

  return (
    <div
      data-testid="dashboard-page"
      className="space-y-6"
    >
      {effectiveTier === 'public_anonymous' && (
        <PublicGuestBanner onSignIn={onSignIn} />
      )}

      {/* Calm Family Archive Greeting Header */}
      <section className="rounded-3xl border border-border bg-surface p-6 sm:p-8 shadow-xs">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6">
          <div className="max-w-2xl space-y-2">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="brand">{t('app.badge')}</Badge>
              {homePersonId && (
                <Badge
                  variant="neutral"
                  icon={<Home className="h-3 w-3" />}
                >
                  {t('tree.homePersonBadge')}
                </Badge>
              )}
            </div>

            <h1 className="font-sans text-2xl sm:text-3xl font-semibold tracking-tight text-primary">
              {user?.first_name
                ? t('onboarding.dashboardGreeting', {
                    name: user.first_name,
                  })
                : instanceTitle || t('app.title')}
            </h1>

            <p className="text-sm sm:text-base text-secondary leading-relaxed">
              {t('app.subtitle')}. {t('app.heroDescription')}
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2.5 shrink-0">
            <Button
              variant="primary"
              size="md"
              leftIcon={<BrandLogoIcon className="h-4 w-4" />}
              onClick={() => navigateTo('/tree')}
            >
              {t('app.openWorkbench')}
            </Button>

            {canEdit && (
              <Button
                variant="secondary"
                size="md"
                leftIcon={<UserPlus className="h-4 w-4" />}
                onClick={() => setQuickAddOpen(true)}
              >
                {t('nav.addPerson')}
              </Button>
            )}
          </div>
        </div>
      </section>

      {/* 4 Live Dashboard Widgets (2x2 Grid) */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-stretch">
        <AnniversariesWidget
          items={dashboard.anniversaries}
          scope={scope}
          onScopeChange={setScope}
          accessTier={
            effectiveTier as
              | 'public_anonymous'
              | 'viewer'
              | 'editor'
              | 'admin'
          }
          onSelectPerson={(personId) => navigateTo(`/persons/${personId}`)}
        />

        <RecentActivityWidget
          transactions={dashboard.recent_activity}
          isAdmin={isAdmin}
          onUndoTransaction={handleUndoTx}
          onOpenHistory={() => navigateTo('/history')}
          onSelectPerson={(personId) => navigateTo(`/persons/${personId}`)}
        />

        <TreeGlanceWidget
          stats={dashboard.stats}
          isAdmin={isAdmin}
          onOpenQuality={() => navigateTo('/quality')}
          onOpenPeople={() => navigateTo('/people')}
          onOpenPlaces={() => navigateTo('/places')}
        />

        <LatestPhotosWidget
          photos={dashboard.latest_photos}
          onPreviewPhoto={(asset) => setPreviewPhoto(asset)}
          onOpenMediaArchive={() => navigateTo('/media')}
        />
      </div>

      <QuickAddRelativeDialog
        open={quickAddOpen}
        onOpenChange={setQuickAddOpen}
        onRelativeAdded={(created) => {
          loadDashboard(scope);
          if (created?.id) {
            navigateTo(`/persons/${created.id}`);
          }
        }}
      />

      <MediaLightboxStudio
        open={Boolean(previewPhoto)}
        onOpenChange={(open) => {
          if (!open) setPreviewPhoto(null);
        }}
        asset={previewPhoto}
        canEdit={canEdit}
        onOpenPerson={(personId) => {
          setPreviewPhoto(null);
          navigateTo(`/persons/${personId}`);
        }}
      />
    </div>
  );
};

export default DashboardPage;
