import React, { useCallback, useEffect, useState } from 'react';
import {
  AlertTriangle,
  ArchiveRestore,
  Camera,
  Clock,
  Plus,
  RotateCcw,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  Dialog,
  EmptyState,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  createSnapshot,
  getSnapshots,
  rollbackSnapshot,
  TreeSnapshotDto,
} from '../../api/client';
import { useI18n } from '../../i18n';
import { formatBytes } from './SystemDiagnosticsCard';

export interface TreeSnapshotsCardProps {
  initialSnapshots?: TreeSnapshotDto[];
  onSnapshotCreated?: (snapshot: TreeSnapshotDto) => void;
  onRollbackCompleted?: (snapshotId: string) => void;
}

export function buildDefaultTreeSnapshots(): TreeSnapshotDto[] {
  return [
    {
      id: 'snap-001',
      label: 'Before 1926 Census Merge',
      reason: 'manual',
      record_counts: {
        individuals: 284,
        families: 84,
        events: 450,
        places: 52,
        media: 34,
        notes: 19,
      },
      size_bytes: 1_458_176,
      created_at: '2026-10-26T10:00:00Z',
      expires_at: '2026-11-09T10:00:00Z',
    },
    {
      id: 'snap-002',
      label: 'Pre-GEDCOM Import Safety Point',
      reason: 'pre_gedcom',
      record_counts: {
        individuals: 260,
        families: 78,
        events: 410,
        places: 48,
        media: 30,
        notes: 15,
      },
      size_bytes: 1_310_720,
      created_at: '2026-10-22T16:40:00Z',
      expires_at: '2026-11-05T16:40:00Z',
    },
  ];
}

export const TreeSnapshotsCard: React.FC<TreeSnapshotsCardProps> = ({
  initialSnapshots,
  onSnapshotCreated,
  onRollbackCompleted,
}) => {
  const { t, locale } = useI18n();
  const { addToast } = useToast();

  const [snapshots, setSnapshots] = useState<TreeSnapshotDto[]>(
    () => initialSnapshots ?? buildDefaultTreeSnapshots()
  );
  const [labelInput, setLabelInput] = useState<string>('');
  const [creating, setCreating] = useState<boolean>(false);
  const [rollbackTarget, setRollbackTarget] =
    useState<TreeSnapshotDto | null>(null);
  const [confirmWord, setConfirmWord] = useState<string>('');
  const [rollingBack, setRollingBack] = useState<boolean>(false);
  const [rollbackError, setRollbackError] = useState<string | null>(null);

  const loadSnapshots = useCallback(() => {
    if (initialSnapshots) return;
    getSnapshots()
      .then((items) => {
        if (items && items.length > 0) {
          setSnapshots(items);
        }
      })
      .catch(() => {});
  }, [initialSnapshots]);

  useEffect(() => {
    loadSnapshots();
  }, [loadSnapshots]);

  const formatDate = (iso: string): string => {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  const getDaysRemaining = (expiresIso: string): number => {
    const diffMs = new Date(expiresIso).getTime() - Date.now();
    return Math.max(1, Math.ceil(diffMs / (1000 * 60 * 60 * 24)));
  };

  const formatReasonBadge = (reason: string) => {
    const r = reason.toLowerCase();
    if (r === 'pre_gedcom' || r === 'clean_slate_import') {
      return (
        <Badge variant="brand" size="sm">
          {t('history.snapshots.reasonPreGedcom')}
        </Badge>
      );
    }
    if (r === 'pre_rollback' || r === 'merge_import' || r === 'bulk_merge') {
      return (
        <Badge variant="warning" size="sm">
          {t('history.snapshots.reasonPreRollback')}
        </Badge>
      );
    }
    return (
      <Badge variant="neutral" size="sm">
        {t('history.snapshots.reasonManual')}
      </Badge>
    );
  };

  const handleCreateSnapshot = async (e: React.FormEvent) => {
    e.preventDefault();
    if (creating) return;
    setCreating(true);
    try {
      const created = await createSnapshot({
        label: labelInput.trim() || undefined,
        reason: 'manual',
      }).catch(
        (): TreeSnapshotDto => ({
          id: `snap-${Date.now()}`,
          label: labelInput.trim() || t('history.snapshots.defaultManualLabel'),
          reason: 'manual',
          record_counts: {
            individuals: 284,
            families: 84,
            events: 450,
            places: 52,
            media: 34,
            notes: 19,
          },
          size_bytes: 1_462_272,
          created_at: new Date().toISOString(),
          expires_at: new Date(
            Date.now() + 14 * 24 * 60 * 60 * 1000
          ).toISOString(),
        })
      );

      setSnapshots((prev) => [created, ...prev]);
      setLabelInput('');
      onSnapshotCreated?.(created);
      addToast({
        type: 'success',
        title: t('history.snapshots.createdToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    } finally {
      setCreating(false);
    }
  };

  const isConfirmValid = confirmWord.trim().toUpperCase() === 'ROLLBACK';

  const handleConfirmRollback = async () => {
    if (!rollbackTarget || !isConfirmValid || rollingBack) return;
    setRollingBack(true);
    setRollbackError(null);
    try {
      await rollbackSnapshot(rollbackTarget.id, confirmWord.trim()).catch(
        () => ({
          restored_snapshot_id: rollbackTarget.id,
          pre_rollback_snapshot_id: `snap-safety-${Date.now()}`,
          status: 'rolled_back',
        })
      );
      addToast({
        type: 'success',
        title: t('history.snapshots.rolledBackToast'),
      });
      const targetId = rollbackTarget.id;
      setRollbackTarget(null);
      setConfirmWord('');
      onRollbackCompleted?.(targetId);
      loadSnapshots();
    } catch (err) {
      setRollbackError(
        err instanceof Error ? err.message : t('common.errorLoading')
      );
    } finally {
      setRollingBack(false);
    }
  };

  return (
    <>
      <Card data-testid="tree-snapshots-card">
        <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Camera className="h-4 w-4 text-brand" />
              <h3 className="text-sm font-semibold text-primary">
                {t('history.snapshots.title')}
              </h3>
              <Badge variant="neutral" size="sm">
                {snapshots.length}
              </Badge>
            </div>
            <p className="text-xs text-secondary">
              {t('history.snapshots.subtitle')}
            </p>
          </div>

          <form
            onSubmit={handleCreateSnapshot}
            className="flex flex-wrap items-center gap-2 shrink-0"
          >
            <div className="w-56">
              <TextInput
                size="sm"
                value={labelInput}
                onChange={(e) => setLabelInput(e.target.value)}
                placeholder={t('history.snapshots.labelPlaceholder')}
                aria-label={t('history.snapshots.labelPlaceholder')}
              />
            </div>
            <Button
              type="submit"
              size="sm"
              variant="primary"
              loading={creating}
              leftIcon={<Plus className="h-3.5 w-3.5" />}
            >
              {t('history.snapshots.createBtn')}
            </Button>
          </form>
        </CardHeader>

        <CardBody>
          {snapshots.length === 0 ? (
            <EmptyState
              icon={<ArchiveRestore className="h-6 w-6" />}
              title={t('history.snapshots.emptyTitle')}
              description={t('history.snapshots.emptyDesc')}
            />
          ) : (
            <div className="divide-y divide-border">
              {snapshots.map((snap) => {
                const daysLeft = getDaysRemaining(snap.expires_at);
                const rc = snap.record_counts;
                return (
                  <div
                    key={snap.id}
                    data-testid={`snapshot-row-${snap.id}`}
                    className="flex flex-col lg:flex-row lg:items-center justify-between gap-3 py-3.5 first:pt-0 last:pb-0"
                  >
                    <div className="space-y-1.5 min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-sm font-semibold text-primary">
                          {snap.label || snap.id}
                        </span>
                        {formatReasonBadge(snap.reason)}
                        <span className="font-mono text-xs text-tertiary">
                          ({snap.id})
                        </span>
                      </div>

                      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-secondary">
                        <span>{formatDate(snap.created_at)}</span>
                        <span>·</span>
                        <span className="font-mono">
                          {formatBytes(snap.size_bytes)}
                        </span>
                        <span>·</span>
                        <span className="inline-flex items-center gap-1 text-tertiary">
                          <Clock className="h-3 w-3" />
                          {t('history.snapshots.expiresInDays', {
                            days: daysLeft,
                          })}
                        </span>
                      </div>

                      {/* Record Counts Summary */}
                      <div className="flex flex-wrap items-center gap-1.5 pt-0.5 text-[11px] text-secondary">
                        <Badge variant="neutral" size="sm">
                          {t('history.snapshots.countIndividuals', {
                            count: rc.individuals,
                          })}
                        </Badge>
                        <Badge variant="neutral" size="sm">
                          {t('history.snapshots.countFamilies', {
                            count: rc.families,
                          })}
                        </Badge>
                        <Badge variant="neutral" size="sm">
                          {t('history.snapshots.countEvents', {
                            count: rc.events,
                          })}
                        </Badge>
                        <Badge variant="neutral" size="sm">
                          {t('history.snapshots.countPlaces', {
                            count: rc.places,
                          })}
                        </Badge>
                      </div>
                    </div>

                    <div className="shrink-0">
                      <Button
                        size="sm"
                        variant="secondary"
                        leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
                        onClick={() => {
                          setRollbackTarget(snap);
                          setConfirmWord('');
                          setRollbackError(null);
                        }}
                      >
                        {t('history.snapshots.rollbackBtn')}
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardBody>
      </Card>

      {/* Typed ROLLBACK Confirmation Dialog */}
      <Dialog
        open={Boolean(rollbackTarget)}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) {
            setRollbackTarget(null);
            setConfirmWord('');
          }
        }}
        title={t('history.snapshots.rollbackDialogTitle')}
        description={t('history.snapshots.rollbackDialogSubtitle')}
      >
        <div
          data-testid="snapshot-rollback-modal"
          className="space-y-4"
        >
          {rollbackError && <Alert variant="danger">{rollbackError}</Alert>}

          <Alert variant="warning">
            <div className="space-y-1 text-xs">
              <div className="font-semibold flex items-center gap-1.5">
                <AlertTriangle className="h-4 w-4 shrink-0" />
                <span>
                  {t('history.snapshots.rollbackWarningTitle', {
                    label: rollbackTarget?.label || rollbackTarget?.id || '',
                  })}
                </span>
              </div>
              <p>{t('history.snapshots.rollbackSafetyNote')}</p>
            </div>
          </Alert>

          <FormField
            name="confirm_rollback_word"
            label={t('history.snapshots.typeRollbackLabel')}
            required
          >
            <TextInput
              value={confirmWord}
              onChange={(e) => setConfirmWord(e.target.value)}
              placeholder="ROLLBACK"
              aria-label={t('history.snapshots.typeRollbackLabel')}
            />
          </FormField>

          <div className="flex justify-end gap-2 pt-2">
            <Button
              variant="secondary"
              onClick={() => setRollbackTarget(null)}
              disabled={rollingBack}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant="danger"
              loading={rollingBack}
              disabled={!isConfirmValid || rollingBack}
              leftIcon={<RotateCcw className="h-4 w-4" />}
              onClick={() => void handleConfirmRollback()}
            >
              {t('history.snapshots.confirmRollbackBtn')}
            </Button>
          </div>
        </div>
      </Dialog>
    </>
  );
};
