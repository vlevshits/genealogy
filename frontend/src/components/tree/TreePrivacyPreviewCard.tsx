import React, { useEffect, useState } from 'react';
import {
  Lock,
  MapPin,
  Calendar,
  Eye,
  Shield,
  UserPlus,
  PenTool,
  Trash2,
  Plus,
  Home,
} from 'lucide-react';
import {
  Avatar,
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  BrandLogoIcon,
  useToast,
} from '../ui';
import {
  DeleteImpactDialog,
  LifeEventDialog,
  PersonEditDialog,
  QuickAddRelativeDialog,
} from '../person';
import { EmptyTreeOnboardingCard } from '../onboarding';
import {
  fetchTreeData,
  QuickAddRelation,
  setHomePerson,
  TreeDataResponse,
  TreePersonNodeDto,
} from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import { PublicGuestBanner } from './PublicGuestBanner';

export interface TreePrivacyPreviewCardProps {
  initialTreeData?: TreeDataResponse;
  onSignIn?: () => void;
  onNavigate?: (path: string) => void;
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
    resolved_home_person_id: 'node-deceased-1',
    nodes,
  };
}

export const TreePrivacyPreviewCard: React.FC<TreePrivacyPreviewCardProps> = ({
  initialTreeData,
  onSignIn,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user, updateUser } = useAuth();
  const { addToast } = useToast();
  const [treeData, setTreeData] = useState<TreeDataResponse>(
    initialTreeData ?? buildDefaultTreePreview('admin')
  );
  const [quickAddState, setQuickAddState] = useState<{
    anchor: TreePersonNodeDto;
    relation: QuickAddRelation;
  } | null>(null);
  const [editTarget, setEditTarget] = useState<TreePersonNodeDto | null>(null);
  const [eventTarget, setEventTarget] = useState<TreePersonNodeDto | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<TreePersonNodeDto | null>(
    null
  );

  const reloadTree = () => {
    fetchTreeData()
      .then((res) => setTreeData(res))
      .catch(() => {});
  };

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

  const canEdit =
    treeData.access_tier === 'editor' || treeData.access_tier === 'admin';
  const isAuthenticatedUser = treeData.access_tier !== 'public_anonymous';
  const resolvedHomePersonId =
    treeData.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

  const handleSetHomePerson = async (node: TreePersonNodeDto) => {
    try {
      const updatedUser = await setHomePerson(node.id);
      updateUser(updatedUser);
      setTreeData((prev) => ({
        ...prev,
        resolved_home_person_id: node.id,
      }));
      addToast({
        type: 'success',
        title: t('tree.homePersonUpdatedToast'),
        description: formatNodeFullName(node),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  if (treeData.nodes.length === 0) {
    return (
      <div className="space-y-4">
        {treeData.access_tier === 'public_anonymous' && (
          <PublicGuestBanner onSignIn={onSignIn} />
        )}
        <EmptyTreeOnboardingCard
          canOnboard={canEdit}
          userRole={
            treeData.access_tier === 'public_anonymous'
              ? 'viewer'
              : treeData.access_tier
          }
          onPersonCreated={() => reloadTree()}
          onImportGedcom={() => {
            if (onNavigate) {
              onNavigate('/settings/gedcom');
            } else if (typeof window !== 'undefined') {
              window.history.pushState({}, '', '/settings/gedcom');
              window.dispatchEvent(new PopStateEvent('popstate'));
            }
          }}
        />
      </div>
    );
  }

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
              const isHomePerson = Boolean(
                resolvedHomePersonId &&
                  (node.id === resolvedHomePersonId ||
                    node.gid === resolvedHomePersonId)
              );
              const isSinglePersonTree = treeData.nodes.length === 1;

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
                      {isHomePerson && (
                        <Badge
                          variant="brand"
                          size="sm"
                          icon={<Home className="h-3 w-3" />}
                        >
                          {t('tree.homePersonBadge')}
                        </Badge>
                      )}
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

                  {isAuthenticatedUser && !isMaskedLiving && (
                    <div className="border-t border-border pt-2.5 space-y-2">
                      {canEdit && (
                        <div className="flex flex-wrap items-center gap-1.5">
                          <Button
                            size="sm"
                            variant="secondary"
                            className={
                              isSinglePersonTree
                                ? 'ring-2 ring-brand/40 animate-pulse'
                                : undefined
                            }
                            leftIcon={<UserPlus className="h-3 w-3" />}
                            onClick={() =>
                              setQuickAddState({
                                anchor: node,
                                relation: 'father',
                              })
                            }
                          >
                            {t('person.actions.addFather')}
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            className={
                              isSinglePersonTree
                                ? 'ring-2 ring-brand/40 animate-pulse'
                                : undefined
                            }
                            leftIcon={<UserPlus className="h-3 w-3" />}
                            onClick={() =>
                              setQuickAddState({
                                anchor: node,
                                relation: 'mother',
                              })
                            }
                          >
                            {t('person.actions.addMother')}
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            leftIcon={<UserPlus className="h-3 w-3" />}
                            onClick={() =>
                              setQuickAddState({
                                anchor: node,
                                relation: 'spouse',
                              })
                            }
                          >
                            {t('person.actions.addSpouse')}
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            leftIcon={<UserPlus className="h-3 w-3" />}
                            onClick={() =>
                              setQuickAddState({
                                anchor: node,
                                relation: 'child',
                              })
                            }
                          >
                            {t('person.actions.addChild')}
                          </Button>
                        </div>
                      )}

                      <div className="flex flex-wrap items-center gap-1.5">
                        {!isHomePerson && (
                          <Button
                            size="sm"
                            variant="ghost"
                            leftIcon={<Home className="h-3 w-3" />}
                            onClick={() => handleSetHomePerson(node)}
                          >
                            {t('tree.setAsHomePerson')}
                          </Button>
                        )}
                        {canEdit && (
                          <>
                            <Button
                              size="sm"
                              variant="ghost"
                              leftIcon={<PenTool className="h-3 w-3" />}
                              onClick={() => setEditTarget(node)}
                            >
                              {t('person.actions.editProfile')}
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              leftIcon={<Plus className="h-3 w-3" />}
                              onClick={() => setEventTarget(node)}
                            >
                              {t('person.actions.addEvent')}
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              leftIcon={
                                <Trash2 className="h-3 w-3 text-danger" />
                              }
                              onClick={() => setDeleteTarget(node)}
                            >
                              {t('person.actions.deletePerson')}
                            </Button>
                          </>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </CardBody>
      </Card>

      <QuickAddRelativeDialog
        open={Boolean(quickAddState)}
        onOpenChange={(open) => {
          if (!open) setQuickAddState(null);
        }}
        anchorPerson={quickAddState?.anchor ?? null}
        relation={quickAddState?.relation ?? 'child'}
        existingPeople={treeData.nodes}
        onRelativeAdded={() => reloadTree()}
      />

      <PersonEditDialog
        open={Boolean(editTarget)}
        onOpenChange={(open) => {
          if (!open) setEditTarget(null);
        }}
        person={editTarget}
        onPersonUpdated={() => reloadTree()}
      />

      <LifeEventDialog
        open={Boolean(eventTarget)}
        onOpenChange={(open) => {
          if (!open) setEventTarget(null);
        }}
        individualId={eventTarget?.id ?? null}
        onEventSaved={() => reloadTree()}
      />

      <DeleteImpactDialog
        open={Boolean(deleteTarget)}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
        person={deleteTarget}
        onPersonDeleted={(deletedId) => {
          setTreeData((prev) => ({
            ...prev,
            nodes: prev.nodes.filter((n) => n.id !== deletedId),
          }));
          reloadTree();
        }}
      />
    </div>
  );
};
