import React, { useEffect, useState } from 'react';
import { Lock, MapPin, Calendar, Eye, Shield } from 'lucide-react';
import {
  Avatar,
  Badge,
  Card,
  CardBody,
  CardHeader,
  BrandLogoIcon,
} from '../ui';
import {
  fetchTreeData,
  TreeDataResponse,
  TreePersonNodeDto,
} from '../../api/client';
import { useI18n } from '../../i18n';
import { PublicGuestBanner } from './PublicGuestBanner';

export interface TreePrivacyPreviewCardProps {
  initialTreeData?: TreeDataResponse;
  onSignIn?: () => void;
}

export function buildDefaultTreePreview(
  accessTier: TreeDataResponse['access_tier'] = 'admin'
): TreeDataResponse {
  const isAnonymous = accessTier === 'public_anonymous';
  const canSeePrivate = accessTier === 'editor' || accessTier === 'admin';

  const nodes: TreePersonNodeDto[] = [
    {
      id: 'node-deceased-1',
      gid: 'I0001',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Alexander',
        patronymic: 'Mikhailovich',
        surname: 'Levshits',
      },
      birth: { date: '1912-03-19', place: 'Vitebsk' },
      death: { date: '1989-11-04', place: 'Leningrad' },
      avatar_url: '/api/v1/media/avatar-01.webp',
      notes: 'Archival founder record.',
    },
    {
      id: 'node-living-1',
      gid: 'I0042',
      sex: 'F',
      is_living: true,
      is_private: false,
      primary_name: {
        given_name: isAnonymous ? 'Living' : 'Elena',
        patronymic: isAnonymous ? null : 'Alexandrovna',
        surname: 'Levshits',
      },
      birth: isAnonymous
        ? null
        : { date: '1994-06-14', place: 'Saint Petersburg, Russia' },
      death: null,
      avatar_url: isAnonymous ? null : '/api/v1/media/avatar-42.webp',
      notes: isAnonymous ? null : 'Living family member.',
    },
  ];

  if (canSeePrivate) {
    nodes.push({
      id: 'node-private-1',
      gid: 'I0099',
      sex: 'M',
      is_living: false,
      is_private: true,
      primary_name: {
        given_name: 'Nikolai',
        patronymic: 'Petrovich',
        surname: 'Morozov',
      },
      birth: { date: '1938-09-02', place: 'Pskov' },
      death: { date: '2004-01-17', place: 'Saint Petersburg' },
      avatar_url: null,
      notes: 'Confidential archival branch record.',
    });
  }

  return {
    access_tier: accessTier,
    enable_public_access: true,
    nodes,
  };
}

export const TreePrivacyPreviewCard: React.FC<TreePrivacyPreviewCardProps> = ({
  initialTreeData,
  onSignIn,
}) => {
  const { t } = useI18n();
  const [treeData, setTreeData] = useState<TreeDataResponse>(
    initialTreeData ?? buildDefaultTreePreview('admin')
  );

  useEffect(() => {
    if (initialTreeData) {
      setTreeData(initialTreeData);
      return;
    }
    let cancelled = false;
    fetchTreeData()
      .then((res) => {
        if (!cancelled) setTreeData(res);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [initialTreeData]);

  const tierLabel =
    treeData.access_tier === 'public_anonymous'
      ? t('tree.privacyPreview.tierPublicAnonymous')
      : treeData.access_tier === 'viewer'
      ? t('tree.privacyPreview.tierViewer')
      : treeData.access_tier === 'editor'
      ? t('tree.privacyPreview.tierEditor')
      : t('tree.privacyPreview.tierAdmin');

  const formatNodeFullName = (node: TreePersonNodeDto): string => {
    const isMaskedLiving =
      node.is_living &&
      (treeData.access_tier === 'public_anonymous' ||
        node.primary_name.given_name === 'Living');
    if (isMaskedLiving) {
      return `${t('tree.privacyPreview.livingMaskedLabel')} ${node.primary_name.surname}`.trim();
    }
    return [
      node.primary_name.given_name,
      node.primary_name.patronymic,
      node.primary_name.surname,
    ]
      .filter(Boolean)
      .join(' ');
  };

  return (
    <div className="space-y-4">
      {treeData.access_tier === 'public_anonymous' && (
        <PublicGuestBanner onSignIn={onSignIn} />
      )}

      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <BrandLogoIcon className="h-5 w-5 text-brand" />
              <h2 className="font-sans text-base sm:text-lg font-semibold tracking-tight text-primary">
                {t('tree.privacyPreview.title')}
              </h2>
            </div>
            <p className="text-xs text-secondary">
              {t('tree.privacyPreview.subtitle')}
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {treeData.access_tier === 'viewer' && (
              <Badge
                variant="neutral"
                icon={<Eye className="h-3.5 w-3.5" />}
              >
                {t('rbac.viewerReadOnlyBadge')}
              </Badge>
            )}
            <Badge
              variant={
                treeData.access_tier === 'public_anonymous'
                  ? 'warning'
                  : 'brand'
              }
              icon={<Shield className="h-3.5 w-3.5" />}
            >
              {tierLabel}
            </Badge>
          </div>
        </CardHeader>

        <CardBody>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {treeData.nodes.map((node) => {
              const isMaskedLiving =
                node.is_living &&
                (treeData.access_tier === 'public_anonymous' ||
                  node.primary_name.given_name === 'Living');
              const fullName = formatNodeFullName(node);

              return (
                <div
                  key={node.id}
                  className="rounded-xl border border-border bg-canvas p-4 space-y-3"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex items-center gap-2.5 min-w-0">
                      <Avatar
                        name={isMaskedLiving ? '?' : fullName}
                        size="md"
                      />
                      <div className="min-w-0">
                        <div className="font-sans text-sm font-semibold text-primary truncate">
                          {fullName}
                        </div>
                        <div className="font-mono text-xs text-tertiary">
                          {node.gid}
                        </div>
                      </div>
                    </div>

                    <div className="flex flex-wrap items-center gap-1 shrink-0">
                      {node.is_private && (
                        <Badge
                          variant="warning"
                          size="sm"
                          icon={<Lock className="h-3 w-3" />}
                        >
                          {t('rbac.privateRecordBadge')}
                        </Badge>
                      )}
                      <Badge
                        variant={node.is_living ? 'brand' : 'neutral'}
                        size="sm"
                      >
                        {node.is_living
                          ? t('tree.privacyPreview.livingBadge')
                          : t('tree.privacyPreview.deceasedBadge')}
                      </Badge>
                    </div>
                  </div>

                  {isMaskedLiving ? (
                    <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface-muted/60 px-2.5 py-1.5 text-xs text-secondary">
                      <Lock className="h-3.5 w-3.5 text-tertiary shrink-0" />
                      <span>{t('tree.privacyPreview.birthHiddenLabel')}</span>
                    </div>
                  ) : (
                    <div className="space-y-1 text-xs text-secondary">
                      {node.birth && (
                        <div className="flex items-center gap-1.5">
                          <Calendar className="h-3.5 w-3.5 text-tertiary shrink-0" />
                          <span>
                            * {node.birth.date}
                            {node.birth.place ? ` · ${node.birth.place}` : ''}
                          </span>
                        </div>
                      )}
                      {node.death && (
                        <div className="flex items-center gap-1.5">
                          <MapPin className="h-3.5 w-3.5 text-tertiary shrink-0" />
                          <span>
                            † {node.death.date}
                            {node.death.place ? ` · ${node.death.place}` : ''}
                          </span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </CardBody>
      </Card>
    </div>
  );
};
