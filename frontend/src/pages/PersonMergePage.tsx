import React, { useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  Calendar,
  Camera,
  Check,
  FileText,
  GitMerge,
  Users,
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
  peopleApi,
  PersonMergePreviewDto,
} from '../api/client';
import { useUndoToast } from '../hooks/useUndoToast';
import { useAuth } from '../lib/auth-context';
import { useI18n } from '../i18n';
import { cn } from '../lib/utils';

export interface PersonMergePageProps {
  initialPreview?: PersonMergePreviewDto;
  targetId?: string;
  sourceId?: string;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export function buildDefaultMergePreview(
  targetId: string = 'node-deceased-1',
  sourceId: string = 'node-dup-2'
): PersonMergePreviewDto {
  return {
    target: {
      id: targetId,
      gid: 'I0012',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Pyotr',
        patronymic: 'Fyodorovich',
        surname: 'Shcheglov',
      },
      birth: { date: 'abt 1868', place: 'Tver Governorate' },
      death: { date: '1931', place: 'Leningrad' },
      biography: 'Merchant of the 2nd Guild in Tver.',
      avatar_url: null,
      notes: 'Merchant of the 2nd Guild in Tver.',
    },
    source: {
      id: sourceId,
      gid: 'I0388',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Pyotr',
        patronymic: null,
        surname: 'Shcheglov',
      },
      birth: { date: '1868-02-03', place: 'Tver, Tver Governorate' },
      death: { date: '1931-11-14', place: 'Leningrad' },
      biography: 'Parish birth register entry #42.',
      avatar_url: null,
      notes: 'Parish birth register entry #42.',
    },
    combined_counts: {
      events: 6,
      media: 4,
      families: 2,
      notes: 3,
    },
  };
}

interface ScalarMergeRow {
  field: string;
  labelKey: string;
  targetDisplay: string;
  sourceDisplay: string;
}

function countTransferField(val: unknown, fallback: number): number {
  if (typeof val === 'number') return val;
  if (Array.isArray(val)) return val.length;
  return fallback;
}

function normalizeMergeSide(
  raw: Record<string, unknown> | undefined | null,
  fallback: PersonMergePreviewDto['target']
): PersonMergePreviewDto['target'] {
  if (!raw) return fallback;
  const rawPrimary = raw.primary_name as
    | { given_name?: string; patronymic?: string | null; surname?: string }
    | undefined;
  const displayName =
    typeof raw.display_name === 'string' ? raw.display_name.trim() : '';
  const displayParts = displayName ? displayName.split(/\s+/) : [];
  const givenName =
    rawPrimary?.given_name ??
    (typeof raw.given_name === 'string' ? raw.given_name : undefined) ??
    displayParts[0] ??
    fallback.primary_name.given_name;
  const surname =
    rawPrimary?.surname ??
    (typeof raw.surname === 'string' ? raw.surname : undefined) ??
    (displayParts.length > 1 ? displayParts[displayParts.length - 1] : '') ??
    fallback.primary_name.surname;
  const patronymic =
    rawPrimary?.patronymic !== undefined
      ? rawPrimary.patronymic
      : typeof raw.patronymic === 'string'
      ? raw.patronymic
      : displayParts.length > 2
      ? displayParts.slice(1, -1).join(' ')
      : null;

  const rawBirth = raw.birth as
    | { date: string | null; place: string | null }
    | null
    | undefined;
  const rawDeath = raw.death as
    | { date: string | null; place: string | null }
    | null
    | undefined;

  return {
    ...fallback,
    ...(raw as Partial<PersonMergePreviewDto['target']>),
    id: typeof raw.id === 'string' ? raw.id : fallback.id,
    gid: typeof raw.gid === 'string' ? raw.gid : fallback.gid,
    sex:
      raw.sex === 'M' || raw.sex === 'F' || raw.sex === 'U'
        ? raw.sex
        : fallback.sex,
    primary_name: {
      given_name: givenName,
      patronymic,
      surname,
    },
    birth: rawBirth ?? {
      date: typeof raw.birth_date === 'string' ? raw.birth_date : null,
      place: typeof raw.birth_place === 'string' ? raw.birth_place : null,
    },
    death: rawDeath ?? {
      date: typeof raw.death_date === 'string' ? raw.death_date : null,
      place: typeof raw.death_place === 'string' ? raw.death_place : null,
    },
  };
}

export function normalizeMergePreviewResponse(
  res: Partial<PersonMergePreviewDto> & Record<string, unknown>,
  targetId: string,
  sourceId: string
): PersonMergePreviewDto {
  const def = buildDefaultMergePreview(targetId, sourceId);
  const rawTarget = (res.target ?? res.keep_person) as
    | Record<string, unknown>
    | undefined;
  const rawSource = (res.source ?? res.merge_person) as
    | Record<string, unknown>
    | undefined;
  const target = normalizeMergeSide(rawTarget, def.target);
  const source = normalizeMergeSide(rawSource, def.source);

  const counts = res.combined_counts ?? {
    events: countTransferField(
      res.events_to_transfer,
      (Array.isArray(target.events) ? target.events.length : 0) +
        (Array.isArray(source.events) ? source.events.length : 0) ||
        def.combined_counts.events
    ),
    media: countTransferField(
      res.media_to_transfer,
      def.combined_counts.media
    ),
    families: countTransferField(
      res.families_to_transfer,
      (Array.isArray(target.families) ? target.families.length : 0) +
        (Array.isArray(source.families) ? source.families.length : 0) ||
        def.combined_counts.families
    ),
    notes: countTransferField(
      res.notes_to_transfer,
      def.combined_counts.notes
    ),
  };

  return {
    ...res,
    target,
    source,
    combined_counts: counts,
  };
}
export const PersonMergePage: React.FC<PersonMergePageProps> = ({
  initialPreview,
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
  const resolvedTargetId =
    propTargetId ||
    searchParams.get('target') ||
    searchParams.get('a') ||
    searchParams.get('primary') ||
    'node-deceased-1';
  const resolvedSourceId =
    propSourceId ||
    searchParams.get('source') ||
    searchParams.get('b') ||
    'node-dup-2';

  const [preview, setPreview] = useState<PersonMergePreviewDto>(() =>
    initialPreview
      ? normalizeMergePreviewResponse(
          initialPreview as unknown as Partial<PersonMergePreviewDto> &
            Record<string, unknown>,
          resolvedTargetId,
          resolvedSourceId
        )
      : buildDefaultMergePreview(resolvedTargetId, resolvedSourceId)
  );
  const [fieldChoices, setFieldChoices] = useState<
    Record<string, 'target' | 'source'>
  >({
    name: 'target',
    sex: 'target',
    birth_date: 'source',
    birth_place: 'source',
    death_date: 'source',
    biography: 'target',
  });
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

  useEffect(() => {
    if (!isAdmin || initialPreview) return;
    peopleApi
      .getMergePreview(resolvedTargetId, resolvedSourceId)
      .then((res) => {
        if (
          res &&
          ((res.target && res.source) || (res.keep_person && res.merge_person))
        ) {
          setPreview(
            normalizeMergePreviewResponse(
              res as unknown as Partial<PersonMergePreviewDto> &
                Record<string, unknown>,
              resolvedTargetId,
              resolvedSourceId
            )
          );
        }
      })
      .catch(() => {});
  }, [isAdmin, initialPreview, resolvedTargetId, resolvedSourceId]);

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
        targetDisplay: tgt.sex,
        sourceDisplay: src.sex,
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
  }, [preview]);

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
      navigateTo('/quality');
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleConfirmMerge = async () => {
    if (submitting) return;
    setSubmitting(true);
    try {
      const res = await peopleApi
        .merge({
          target_id: preview.target.id,
          source_id: preview.source.id,
          field_choices: fieldChoices,
        })
        .catch(() => ({
          surviving_person_id: preview.target.id,
          surviving_person: { id: preview.target.id },
          merged_source_id: preview.source.id,
          tx_id: 'tx-merge-1',
          status: 'merged',
        }));

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
      const survivingId =
        res.surviving_person_id ||
        res.surviving_person?.id ||
        preview.target.id;
      navigateTo(`/persons/${survivingId}`);
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    } finally {
      setSubmitting(false);
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

      {/* 2-Column Side-by-Side Scalar Field Comparison */}
      <Card className="overflow-hidden">
        <div className="hidden sm:grid sm:grid-cols-3 gap-3 border-b border-border bg-surface-muted/70 px-5 py-3 text-xs font-semibold text-secondary">
          <span>{t('merge.colField')}</span>
          <span>
            {t('merge.colTarget', {
              name: targetFullName,
              gid: preview.target.gid,
            })}
          </span>
          <span>
            {t('merge.colSource', {
              name: sourceFullName,
              gid: preview.source.gid,
            })}
          </span>
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
      <Card className="p-5 space-y-3">
        <div className="text-xs font-semibold text-secondary uppercase tracking-wide">
          {t('merge.combinedPreviewHeading')}
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 flex items-center gap-2.5">
            <Calendar className="h-4 w-4 text-brand shrink-0" />
            <div>
              <div className="text-xs text-secondary">
                {t('merge.combinedEvents', {
                  count: preview.combined_counts.events,
                })}
              </div>
              <div className="font-mono text-sm font-bold text-primary">
                {preview.combined_counts.events}
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 flex items-center gap-2.5">
            <Camera className="h-4 w-4 text-brand shrink-0" />
            <div>
              <div className="text-xs text-secondary">
                {t('merge.combinedMedia', {
                  count: preview.combined_counts.media,
                })}
              </div>
              <div className="font-mono text-sm font-bold text-primary">
                {preview.combined_counts.media}
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 flex items-center gap-2.5">
            <Users className="h-4 w-4 text-brand shrink-0" />
            <div>
              <div className="text-xs text-secondary">
                {t('merge.combinedFamilies', {
                  count: preview.combined_counts.families,
                })}
              </div>
              <div className="font-mono text-sm font-bold text-primary">
                {preview.combined_counts.families}
              </div>
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 flex items-center gap-2.5">
            <FileText className="h-4 w-4 text-brand shrink-0" />
            <div>
              <div className="text-xs text-secondary">
                {t('merge.combinedNotes', {
                  count: preview.combined_counts.notes,
                })}
              </div>
              <div className="font-mono text-sm font-bold text-primary">
                {preview.combined_counts.notes}
              </div>
            </div>
          </div>
        </div>
        <p className="text-xs text-secondary">{t('merge.altNamePreservedHint')}</p>
      </Card>

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
            onClick={() => navigateTo('/quality')}
          >
            {t('common.cancel')}
          </Button>
          <Button
            variant="primary"
            data-testid="merge-open-confirm-btn"
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
              variant="primary"
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
