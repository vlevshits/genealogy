import React, { useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  Check,
  ExternalLink,
  GitMerge,
  XCircle,
} from 'lucide-react';
import {
  AccessDeniedCard,
  Alert,
  Badge,
  Button,
  Card,
  Dialog,
  PageHeader,
  useToast,
} from '../components/ui';
import {
  getTree,
  notifyTreeDataChanged,
  peopleApi,
  PersonMergePreviewDto,
  TreePersonNodeDto,
} from '../api/client';
import { useUndoToast } from '../hooks/useUndoToast';
import { useAuth } from '../lib/auth-context';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';
import { cn } from '../lib/utils';

import {
  buildDefaultMergePreview,
  buildMergePreviewFromTreeNodes,
  computeDefaultFieldChoices,
  normalizeMergePreviewResponse,
  rankMergeCandidatesForTarget,
  ScalarMergeRow,
} from '../components/person/person-merge-helpers';
import {
  PersonMergeCombinedUnionCard,
  PersonMergeSelectorBar,
} from '../components/person/PersonMergeSelectorBar';

export {
  buildDefaultMergePreview,
  buildMergePreviewFromTreeNodes,
  computeDefaultFieldChoices,
  normalizeMergePreviewResponse,
  rankMergeCandidatesForTarget,
};

export interface PersonMergePageProps {
  initialPreview?: PersonMergePreviewDto;
  initialPeople?: TreePersonNodeDto[];
  targetId?: string;
  sourceId?: string;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export const PersonMergePage: React.FC<PersonMergePageProps> = ({
  initialPreview,
  initialPeople = [],
  targetId: propTargetId,
  sourceId: propSourceId,
  userRole,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();
  const { showUndoToast } = useUndoToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isAdmin = effectiveRole === 'admin';

  const searchParams =
    typeof window !== 'undefined'
      ? new URLSearchParams(window.location.search)
      : new URLSearchParams();
  const explicitTargetParam =
    propTargetId ||
    searchParams.get('target') ||
    searchParams.get('a') ||
    searchParams.get('primary') ||
    '';
  const explicitSourceParam =
    propSourceId ||
    searchParams.get('source') ||
    searchParams.get('b') ||
    '';
  const explicitReturnTo = searchParams.get('returnTo');
  const returnTo = explicitReturnTo || '/quality?tab=duplicates';

  const resolvedInitialTargetId = explicitTargetParam || 'node-deceased-1';
  const resolvedInitialSourceId = explicitSourceParam || 'node-dup-2';

  const [people, setPeople] = useState<TreePersonNodeDto[]>(initialPeople);
  const [activeTargetId, setActiveTargetId] = useState<string>(
    resolvedInitialTargetId
  );
  const [activeSourceId, setActiveSourceId] = useState<string>(
    resolvedInitialSourceId
  );

  const [preview, setPreview] = useState<PersonMergePreviewDto>(() =>
    initialPreview
      ? normalizeMergePreviewResponse(
        initialPreview as unknown as Partial<PersonMergePreviewDto> &
        Record<string, unknown>,
        resolvedInitialTargetId,
        resolvedInitialSourceId
      )
      : buildDefaultMergePreview(
        resolvedInitialTargetId,
        resolvedInitialSourceId
      )
  );
  const [fieldChoices, setFieldChoices] = useState<
    Record<string, 'target' | 'source'>
  >(() => computeDefaultFieldChoices(preview));
  const [confirmOpen, setConfirmOpen] = useState<boolean>(false);
  const [submitting, setSubmitting] = useState<boolean>(false);

  const navigateTo = (nextPath: string) => {
    if (onNavigate) {
      onNavigate(nextPath);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', nextPath);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  const syncUrlParams = (nextTarget: string, nextSource: string) => {
    if (typeof window === 'undefined') return;
    const params = new URLSearchParams(window.location.search);
    params.set('target', nextTarget);
    params.set('source', nextSource);
    if (explicitReturnTo) {
      params.set('returnTo', explicitReturnTo);
    }
    const nextUrl = `${window.location.pathname}?${params.toString()}`;
    window.history.replaceState({}, '', nextUrl);
  };

  // Load real tree profiles on mount so user can search/select any profile in the tree
  useEffect(() => {
    if (!isAdmin) return;
    if (initialPeople.length > 0) {
      const nodes = initialPeople;
      const tgtNode =
        nodes.find(
          (n) =>
            n.id === explicitTargetParam || n.gid === explicitTargetParam
        ) ?? nodes[0];
      const resolvedTgtId = tgtNode ? tgtNode.id : resolvedInitialTargetId;
      let resolvedSrcId = resolvedInitialSourceId;
      if (!explicitSourceParam && nodes.length > 1) {
        const ranked = rankMergeCandidatesForTarget(tgtNode, nodes);
        if (ranked.length > 0) {
          resolvedSrcId = ranked[0].person.id;
        }
      }
      setActiveTargetId(resolvedTgtId);
      setActiveSourceId(resolvedSrcId);
      return;
    }

    getTree({ mode: 'all' })
      .then((res) => {
        const nodes = res?.nodes ?? [];
        if (nodes.length === 0) return;
        setPeople(nodes);

        const tgtNode =
          nodes.find(
            (n) =>
              n.id === explicitTargetParam || n.gid === explicitTargetParam
          ) ?? nodes[0];
        const resolvedTgtId = tgtNode ? tgtNode.id : resolvedInitialTargetId;

        let resolvedSrcId = explicitSourceParam;
        if (!resolvedSrcId || resolvedSrcId === 'node-dup-2') {
          const ranked = rankMergeCandidatesForTarget(tgtNode, nodes);
          if (ranked.length > 0) {
            resolvedSrcId = ranked[0].person.id;
          }
        } else {
          const matchedSrc = nodes.find(
            (n) => n.id === resolvedSrcId || n.gid === resolvedSrcId
          );
          if (matchedSrc) {
            resolvedSrcId = matchedSrc.id;
          }
        }

        if (resolvedTgtId) setActiveTargetId(resolvedTgtId);
        if (resolvedSrcId) setActiveSourceId(resolvedSrcId);
        if (resolvedTgtId && resolvedSrcId) {
          syncUrlParams(resolvedTgtId, resolvedSrcId);
        }
      })
      .catch(() => { });
  }, [isAdmin]);

  useEffect(() => {
    if (
      initialPreview &&
      activeTargetId === resolvedInitialTargetId &&
      activeSourceId === resolvedInitialSourceId
    ) {
      const normalized = normalizeMergePreviewResponse(
        initialPreview as unknown as Partial<PersonMergePreviewDto> &
        Record<string, unknown>,
        activeTargetId,
        activeSourceId
      );
      setPreview(normalized);
      setFieldChoices(computeDefaultFieldChoices(normalized));
      return;
    }
    if (!isAdmin || !activeTargetId || !activeSourceId) return;

    const targetNode = people.find(
      (n) => n.id === activeTargetId || n.gid === activeTargetId
    );
    const sourceNode = people.find(
      (n) => n.id === activeSourceId || n.gid === activeSourceId
    );

    if (targetNode || sourceNode) {
      const treeBased = buildMergePreviewFromTreeNodes(
        targetNode,
        sourceNode,
        activeTargetId,
        activeSourceId
      );
      setPreview(treeBased);
      setFieldChoices(computeDefaultFieldChoices(treeBased));
    }

    if (activeTargetId === activeSourceId) return;

    peopleApi
      .getMergePreview(activeTargetId, activeSourceId)
      .then((res) => {
        if (
          res &&
          ((res.target && res.source) || (res.keep_person && res.merge_person))
        ) {
          const normalized = normalizeMergePreviewResponse(
            res as unknown as Partial<PersonMergePreviewDto> &
            Record<string, unknown>,
            activeTargetId,
            activeSourceId
          );
          setPreview(normalized);
          setFieldChoices(computeDefaultFieldChoices(normalized));
        }
      })
      .catch(() => { });
  }, [
    isAdmin,
    initialPreview,
    activeTargetId,
    activeSourceId,
    people,
    resolvedInitialTargetId,
    resolvedInitialSourceId,
  ]);

  const handleSelectTarget = (nextTargetId: string) => {
    if (!nextTargetId) return;
    if (nextTargetId === activeSourceId) {
      // Swap automatically if user selects the current source as target
      const prevTarget = activeTargetId;
      setActiveTargetId(nextTargetId);
      setActiveSourceId(prevTarget);
      syncUrlParams(nextTargetId, prevTarget);
      return;
    }
    setActiveTargetId(nextTargetId);
    syncUrlParams(nextTargetId, activeSourceId);
  };

  const handleSelectSource = (nextSourceId: string) => {
    if (!nextSourceId) return;
    if (nextSourceId === activeTargetId) {
      const prevSource = activeSourceId;
      setActiveSourceId(nextSourceId);
      setActiveTargetId(prevSource);
      syncUrlParams(prevSource, nextSourceId);
      return;
    }
    setActiveSourceId(nextSourceId);
    syncUrlParams(activeTargetId, nextSourceId);
  };

  const handleSwapDirection = () => {
    const nextTarget = activeSourceId;
    const nextSource = activeTargetId;
    setActiveTargetId(nextTarget);
    setActiveSourceId(nextSource);
    syncUrlParams(nextTarget, nextSource);
  };

  const rows = useMemo<ScalarMergeRow[]>(() => {
    const tgt = preview.target;
    const src = preview.source;
    const tgtName = [
      tgt.primary_name.given_name,
      tgt.primary_name.patronymic,
      tgt.primary_name.surname,
    ]
      .filter(Boolean)
      .join(' ');
    const srcName = [
      src.primary_name.given_name,
      src.primary_name.patronymic,
      src.primary_name.surname,
    ]
      .filter(Boolean)
      .join(' ');

    const formatSexLabel = (sex: string | undefined): string => {
      if (sex === 'M' || sex === 'male') return `${t('person.sex.male')} (M)`;
      if (sex === 'F' || sex === 'female') return `${t('person.sex.female')} (F)`;
      if (sex === 'U' || sex === 'unknown') return `${t('person.sex.unknown')} (U)`;
      return sex || '—';
    };

    return [
      {
        field: 'name',
        labelKey: 'merge.fields.name',
        targetDisplay: tgtName || '—',
        sourceDisplay: srcName || '—',
      },
      {
        field: 'sex',
        labelKey: 'merge.fields.sex',
        targetDisplay: formatSexLabel(tgt.sex),
        sourceDisplay: formatSexLabel(src.sex),
      },
      {
        field: 'birth_date',
        labelKey: 'merge.fields.birthDate',
        targetDisplay: tgt.birth?.date || '—',
        sourceDisplay: src.birth?.date || '—',
      },
      {
        field: 'birth_place',
        labelKey: 'merge.fields.birthPlace',
        targetDisplay: tgt.birth?.place || '—',
        sourceDisplay: src.birth?.place || '—',
      },
      {
        field: 'death_date',
        labelKey: 'merge.fields.deathDate',
        targetDisplay: tgt.death?.date || '—',
        sourceDisplay: src.death?.date || '—',
      },
      {
        field: 'biography',
        labelKey: 'merge.fields.biography',
        targetDisplay: tgt.biography || tgt.notes || '—',
        sourceDisplay: src.biography || src.notes || '—',
      },
    ];
  }, [preview, t]);

  if (!isAdmin) {
    return (
      <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-12">
        <AccessDeniedCard
          currentRole={effectiveRole === 'editor' ? 'editor' : 'viewer'}
          onOpenProfile={() => navigateTo('/settings/profile')}
          onBackToTree={() => navigateTo('/tree')}
        />
      </div>
    );
  }

  const targetFullName = [
    preview.target.primary_name.given_name,
    preview.target.primary_name.patronymic,
    preview.target.primary_name.surname,
  ]
    .filter(Boolean)
    .join(' ');

  const sourceFullName = [
    preview.source.primary_name.given_name,
    preview.source.primary_name.patronymic,
    preview.source.primary_name.surname,
  ]
    .filter(Boolean)
    .join(' ');

  const handleMarkNotDuplicate = async () => {
    try {
      await peopleApi
        .markNotDuplicate(preview.target.id, preview.source.id)
        .catch(() => ({ status: 'ok' }));
      addToast({
        type: 'success',
        title: t('quality.duplicates.markedNotDuplicateToast'),
      });
      navigateTo(returnTo);
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleConfirmMerge = async () => {
    if (submitting) return;
    setSubmitting(true);
    try {
      const res = await peopleApi.merge({
        target_id: preview.target.id,
        source_id: preview.source.id,
        field_choices: fieldChoices,
      });

      if (res.tx_id) {
        showUndoToast({
          txId: res.tx_id,
          title: t('history.undo_toast_label'),
          description: t('merge.successToast', { name: targetFullName }),
        });
      } else {
        addToast({
          type: 'success',
          title: t('merge.successToast', { name: targetFullName }),
        });
      }
      setConfirmOpen(false);
      notifyTreeDataChanged();
      const destination =
        explicitReturnTo || `/persons/${encodeURIComponent(preview.target.id)}`;
      navigateTo(destination);
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setSubmitting(false);
    }
  };

  const handleOpenProfile = (
    e: React.MouseEvent<HTMLButtonElement>,
    personId: string
  ) => {
    e.stopPropagation();
    const url = `/persons/${personId}`;
    if (typeof window !== 'undefined') {
      window.open(url, '_blank', 'noopener,noreferrer');
    }
  };

  const handleAuxClickProfile = (
    e: React.MouseEvent<HTMLButtonElement>,
    personId: string
  ) => {
    if (e.button === 1) {
      e.stopPropagation();
      const url = `/persons/${personId}`;
      if (typeof window !== 'undefined') {
        window.open(url, '_blank', 'noopener,noreferrer');
      }
    }
  };

  return (
    <div
      data-testid="person-merge-page"
      className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6"
    >
      <PageHeader
        title={t('merge.pageTitle')}
        subtitle={t('merge.pageSubtitle')}
        secondaryActions={
          <Badge variant="brand">
            {t('merge.survivingIdBadge', { gid: preview.target.gid })}
          </Badge>
        }
      />

      {/* Profile Selector & Swap Bar */}
      <PersonMergeSelectorBar
        people={people}
        targetId={activeTargetId}
        sourceId={activeSourceId}
        onSelectTarget={handleSelectTarget}
        onSelectSource={handleSelectSource}
        onSwapDirection={handleSwapDirection}
      />

      {/* Responsive Profile Quick-Inspect Chips */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
        <Card className="p-3.5 sm:p-4 border-brand/30 bg-brand/5 flex items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[11px] font-semibold text-brand uppercase tracking-wider">
                {t('merge.colTargetShort')}
              </span>
              <Badge variant="brand" size="sm">
                {preview.target.gid}
              </Badge>
            </div>
            <p className="text-sm font-semibold text-primary truncate">
              {targetFullName || '—'}
            </p>
          </div>
          <Button
            variant="secondary"
            size="xs"
            rightIcon={<ExternalLink className="h-3 w-3 shrink-0" />}
            onClick={(e) => handleOpenProfile(e, preview.target.id)}
            onAuxClick={(e) => handleAuxClickProfile(e, preview.target.id)}
            aria-label={t('merge.openTargetProfileAria', { name: targetFullName })}
            title={t('merge.openTargetProfileAria', { name: targetFullName })}
            data-testid="merge-target-open-profile-chip-btn"
            data-url={`/persons/${preview.target.id}`}
            className="shrink-0"
          >
            {t('merge.viewProfileBtn')}
          </Button>
        </Card>

        <Card className="p-3.5 sm:p-4 flex items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[11px] font-semibold text-secondary uppercase tracking-wider">
                {t('merge.colSourceShort')}
              </span>
              <Badge variant="outline" size="sm">
                {preview.source.gid}
              </Badge>
            </div>
            <p className="text-sm font-semibold text-primary truncate">
              {sourceFullName || '—'}
            </p>
          </div>
          <Button
            variant="secondary"
            size="xs"
            rightIcon={<ExternalLink className="h-3 w-3 shrink-0" />}
            onClick={(e) => handleOpenProfile(e, preview.source.id)}
            onAuxClick={(e) => handleAuxClickProfile(e, preview.source.id)}
            aria-label={t('merge.openSourceProfileAria', { name: sourceFullName })}
            title={t('merge.openSourceProfileAria', { name: sourceFullName })}
            data-testid="merge-source-open-profile-chip-btn"
            data-url={`/persons/${preview.source.id}`}
            className="shrink-0"
          >
            {t('merge.viewProfileBtn')}
          </Button>
        </Card>
      </div>

      {/* 2-Column Side-by-Side Scalar Field Comparison */}
      <Card className="overflow-hidden">
        <div className="hidden sm:grid sm:grid-cols-3 gap-3 border-b border-border bg-surface-muted/70 px-5 py-3 text-xs font-semibold text-secondary items-center">
          <span>{t('merge.colField')}</span>
          <div className="flex items-center justify-between gap-2 min-w-0">
            <span className="truncate">
              {t('merge.colTarget', {
                name: targetFullName,
                gid: preview.target.gid,
              })}
            </span>
            <Button
              variant="ghost"
              size="xs"
              rightIcon={<ExternalLink className="h-3 w-3 shrink-0" />}
              onClick={(e) => handleOpenProfile(e, preview.target.id)}
              onAuxClick={(e) => handleAuxClickProfile(e, preview.target.id)}
              aria-label={t('merge.openTargetProfileAria', { name: targetFullName })}
              title={t('merge.openTargetProfileAria', { name: targetFullName })}
              data-testid="merge-target-open-profile-btn"
              data-url={`/persons/${preview.target.id}`}
              className="shrink-0 text-secondary hover:text-primary"
            >
              {t('merge.viewProfileShort')}
            </Button>
          </div>
          <div className="flex items-center justify-between gap-2 min-w-0">
            <span className="truncate">
              {t('merge.colSource', {
                name: sourceFullName,
                gid: preview.source.gid,
              })}
            </span>
            <Button
              variant="ghost"
              size="xs"
              rightIcon={<ExternalLink className="h-3 w-3 shrink-0" />}
              onClick={(e) => handleOpenProfile(e, preview.source.id)}
              onAuxClick={(e) => handleAuxClickProfile(e, preview.source.id)}
              aria-label={t('merge.openSourceProfileAria', { name: sourceFullName })}
              title={t('merge.openSourceProfileAria', { name: sourceFullName })}
              data-testid="merge-source-open-profile-btn"
              data-url={`/persons/${preview.source.id}`}
              className="shrink-0 text-secondary hover:text-primary"
            >
              {t('merge.viewProfileShort')}
            </Button>
          </div>
        </div>

        <div className="divide-y divide-border">
          {rows.map((row) => {
            const choice = fieldChoices[row.field] ?? 'target';
            const label = t(row.labelKey);
            return (
              <div
                key={row.field}
                role="radiogroup"
                aria-label={label}
                data-testid={`merge-row-${row.field}`}
                className="grid grid-cols-1 sm:grid-cols-3 gap-3 p-4 items-center text-xs"
              >
                <div className="font-semibold text-primary">{label}</div>

                <div
                  role="radio"
                  aria-checked={choice === 'target'}
                  aria-label={`${t('merge.colTargetShort')}: ${row.targetDisplay}`}
                  tabIndex={0}
                  data-testid={`merge-pick-target-${row.field}`}
                  onClick={() =>
                    setFieldChoices((prev) => ({
                      ...prev,
                      [row.field]: 'target',
                    }))
                  }
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      setFieldChoices((prev) => ({
                        ...prev,
                        [row.field]: 'target',
                      }));
                    }
                  }}
                  className={cn(
                    'flex items-center justify-between gap-2 rounded-xl border p-3 cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                    choice === 'target'
                      ? 'border-brand bg-brand/10 text-primary font-medium'
                      : 'border-border bg-canvas text-secondary hover:border-brand/40'
                  )}
                >
                  <div className="min-w-0">
                    <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wider text-tertiary mb-1 block">
                      {t('merge.colTargetShort')}
                    </span>
                    <span className="break-words">{row.targetDisplay}</span>
                  </div>
                  {choice === 'target' && (
                    <Check className="h-4 w-4 text-brand shrink-0" />
                  )}
                </div>

                <div
                  role="radio"
                  aria-checked={choice === 'source'}
                  aria-label={`${t('merge.colSourceShort')}: ${row.sourceDisplay}`}
                  tabIndex={0}
                  data-testid={`merge-pick-source-${row.field}`}
                  onClick={() =>
                    setFieldChoices((prev) => ({
                      ...prev,
                      [row.field]: 'source',
                    }))
                  }
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      setFieldChoices((prev) => ({
                        ...prev,
                        [row.field]: 'source',
                      }));
                    }
                  }}
                  className={cn(
                    'flex items-center justify-between gap-2 rounded-xl border p-3 cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                    choice === 'source'
                      ? 'border-brand bg-brand/10 text-primary font-medium'
                      : 'border-border bg-canvas text-secondary hover:border-brand/40'
                  )}
                >
                  <div className="min-w-0">
                    <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wider text-tertiary mb-1 block">
                      {t('merge.colSourceShort')}
                    </span>
                    <span className="break-words">{row.sourceDisplay}</span>
                  </div>
                  {choice === 'source' && (
                    <Check className="h-4 w-4 text-brand shrink-0" />
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </Card>

      {/* Combined Union Preview Card */}
      <PersonMergeCombinedUnionCard counts={preview.combined_counts} />

      {/* Action Footer */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Button
          variant="secondary"
          data-testid="merge-not-duplicate-btn"
          leftIcon={<XCircle className="h-4 w-4" />}
          onClick={() => void handleMarkNotDuplicate()}
        >
          {t('quality.duplicates.notDuplicateBtn')}
        </Button>

        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={() => navigateTo(returnTo)}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            data-testid="merge-open-confirm-btn"
            disabled={preview.target.id === preview.source.id}
            leftIcon={<GitMerge className="h-4 w-4" />}
            onClick={() => setConfirmOpen(true)}
          >
            {t('merge.confirmMergeBtn')}
          </Button>
        </div>
      </div>

      {/* Confirmation Modal */}
      <Dialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        size="sm"
        title={t('merge.confirmDialogTitle')}
        description={t('merge.confirmDialogDesc', {
          target: targetFullName,
          source: sourceFullName,
          gid: preview.target.gid,
        })}
      >
        <div className="space-y-4">
          <Alert
            variant="warning"
            icon={<AlertTriangle className="h-4 w-4 text-warning shrink-0" />}
          >
            {t('merge.undoableNotice')}
          </Alert>

          <div className="flex justify-end gap-2">
            <Button
              variant="secondary"
              onClick={() => setConfirmOpen(false)}
              disabled={submitting}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant="danger"
              loading={submitting}
              data-testid="merge-execute-confirm-btn"
              leftIcon={<GitMerge className="h-4 w-4" />}
              onClick={() => void handleConfirmMerge()}
            >
              {t('merge.confirmMergeBtn')}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};

export default PersonMergePage;
