import React, { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, GitMerge, RefreshCw, Save } from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Dialog,
  RadioChoiceRow,
} from '../ui';
import { useI18n } from '../../i18n';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import {
  HistoryTranslateFn,
  formatDiffValue,
  formatFieldLabel,
} from './history-diff-utils';

export interface ConflictFieldEntry {
  field: string;
  label?: string;
  myValue: unknown;
  theirValue: unknown;
}

export interface ConflictResolutionModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  updatedByName?: string | null;
  updatedAt?: string | null;
  remoteVersion?: number;
  myChanges: Record<string, unknown>;
  theirData?: Record<string, unknown>;
  theirChanges?: Record<string, unknown>;
  fieldLabels?: Record<string, string>;
  onKeepMine: (remoteVersion: number) => void | Promise<void>;
  onUseTheirs: (theirData: Record<string, unknown>) => void;
  onMergeFields?: (
    mergedValues: Record<string, unknown>,
    remoteVersion: number
  ) => void | Promise<void>;
  onMerge?: (
    mergedValues: Record<string, unknown>,
    remoteVersion: number
  ) => void | Promise<void>;
}

export function computeConflictFields(
  myChanges: Record<string, unknown>,
  theirData: Record<string, unknown>,
  fieldLabels: Record<string, string> = {},
  t?: HistoryTranslateFn
): ConflictFieldEntry[] {
  const keys = Object.keys(myChanges).filter(
    (k) => k !== 'version' && k !== 'updated_at'
  );
  const list: ConflictFieldEntry[] = [];
  for (const key of keys) {
    const mine = myChanges[key];
    const theirs = theirData[key];
    if (JSON.stringify(mine) !== JSON.stringify(theirs)) {
      list.push({
        field: key,
        label: fieldLabels[key] || formatFieldLabel(key, t ?? ((k) => k)),
        myValue: mine,
        theirValue: theirs,
      });
    }
  }
  return list;
}

function formatConflictTimeLabel(
  updatedAt: string | null | undefined,
  remoteVersion: number,
  locale: 'en' | 'ru'
): string {
  const trimmed = updatedAt?.trim();
  if (!trimmed) return `v${remoteVersion}`;
  const parsed = new Date(trimmed);
  if (!Number.isNaN(parsed.getTime())) {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(parsed);
    } catch {
      return trimmed;
    }
  }
  return trimmed;
}

export const ConflictResolutionModal: React.FC<
  ConflictResolutionModalProps
> = ({
  open,
  onOpenChange,
  updatedByName,
  updatedAt,
  remoteVersion = 2,
  myChanges,
  theirData,
  theirChanges,
  fieldLabels = {},
  onKeepMine,
  onUseTheirs,
  onMergeFields,
  onMerge,
}) => {
  const { t, locale } = useI18n();
  const [mergeMode, setMergeMode] = useState<boolean>(false);
  const [submitting, setSubmitting] = useState<boolean>(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const resolvedTheirs = useMemo(
    () => theirData ?? theirChanges ?? {},
    [theirData, theirChanges]
  );

  const differingFields = useMemo<ConflictFieldEntry[]>(
    () => computeConflictFields(myChanges, resolvedTheirs, fieldLabels, t),
    [myChanges, resolvedTheirs, fieldLabels, t]
  );

  const [selectedSourceByField, setSelectedSourceByField] = useState<
    Record<string, 'mine' | 'theirs'>
  >({});

  useEffect(() => {
    setMergeMode(false);
    setSelectedSourceByField({});
    setSubmitError(null);
    setSubmitting(false);
  }, [open, myChanges, theirData, theirChanges, remoteVersion]);

  const getChoice = (field: string): 'mine' | 'theirs' =>
    selectedSourceByField[field] ?? 'mine';

  const handleApplyFieldMerge = async () => {
    if (submitting) return;
    const merged: Record<string, unknown> = { ...myChanges };
    for (const item of differingFields) {
      const choice = getChoice(item.field);
      merged[item.field] =
        choice === 'mine' ? item.myValue : item.theirValue;
    }
    setSubmitting(true);
    setSubmitError(null);
    try {
      await onMergeFields?.(merged, remoteVersion);
      await onMerge?.(merged, remoteVersion);
    } catch (err) {
      setSubmitError(getLocalizedErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  const handleKeepMine = async () => {
    if (submitting) return;
    setSubmitting(true);
    setSubmitError(null);
    try {
      await onKeepMine(remoteVersion);
    } catch (err) {
      setSubmitError(getLocalizedErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  const handleUseTheirs = async () => {
    if (submitting) return;
    setSubmitting(true);
    setSubmitError(null);
    try {
      await onUseTheirs(resolvedTheirs);
    } catch (err) {
      setSubmitError(getLocalizedErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  const editorName = updatedByName || t('history.conflict.anotherEditor');

  const modalFooter = (
    <div className="flex w-full flex-col-reverse sm:flex-row sm:items-center sm:justify-between gap-2">
      <Button
        variant="secondary"
        size="md"
        disabled={submitting}
        leftIcon={<RefreshCw className="h-4 w-4" />}
        onClick={() => void handleUseTheirs()}
        className="w-full sm:w-auto"
      >
        {t('history.conflict.useTheirsBtn')}
      </Button>

      <div className="flex flex-col sm:flex-row sm:items-center gap-2">
        {!mergeMode ? (
          <Button
            variant="secondary"
            size="md"
            disabled={submitting}
            leftIcon={<GitMerge className="h-4 w-4" />}
            onClick={() => setMergeMode(true)}
            className="w-full sm:w-auto"
          >
            {t('history.conflict.mergeFieldByFieldBtn')}
          </Button>
        ) : (
          <Button
            variant="primary"
            size="md"
            loading={submitting}
            leftIcon={<GitMerge className="h-4 w-4" />}
            onClick={() => void handleApplyFieldMerge()}
            className="w-full sm:w-auto"
          >
            {t('history.conflict.applyMergedBtn')}
          </Button>
        )}

        <Button
          variant={mergeMode ? 'secondary' : 'primary'}
          size="md"
          loading={submitting && !mergeMode}
          disabled={submitting}
          leftIcon={<Save className="h-4 w-4" />}
          onClick={() => void handleKeepMine()}
          className="w-full sm:w-auto"
        >
          {t('history.conflict.keepMineBtn')}
        </Button>
      </div>
    </div>
  );

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      data-testid="conflict-resolution-modal"
      title={t('history.conflict.title')}
      description={t('history.conflict.subtitle', { editor: editorName })}
      footer={modalFooter}
    >
      <div className="space-y-4">
        <Alert variant="warning">
          <div className="flex items-center justify-between gap-2">
            <span>
              {t('history.conflict.bannerText', {
                name: editorName,
                editor: editorName,
                time: formatConflictTimeLabel(updatedAt, remoteVersion, locale),
                version: remoteVersion,
              })}
            </span>
            <Badge variant="warning" size="sm">
              v{remoteVersion}
            </Badge>
          </div>
        </Alert>

        {submitError && (
          <Alert variant="danger" data-testid="conflict-error-alert">
            {submitError}
          </Alert>
        )}

        {mergeMode && (
          <p className="text-xs text-secondary flex items-center gap-1.5">
            <AlertTriangle className="h-3.5 w-3.5 text-brand shrink-0" />
            <span>{t('history.conflict.mergeInstruction')}</span>
          </p>
        )}

        {/* Field-by-field comparison table / interactive radio cards */}
        <div className="rounded-2xl border border-border bg-surface overflow-hidden">
          <div className="hidden sm:grid sm:grid-cols-3 gap-2 border-b border-border bg-surface-muted/70 px-4 py-2.5 text-xs font-semibold text-secondary">
            <span>{t('history.conflict.colField')}</span>
            <span>{t('history.conflict.colMine')}</span>
            <span>{t('history.conflict.colTheirs', { editor: editorName })}</span>
          </div>

          <div className="divide-y divide-border">
            {differingFields.length === 0 ? (
              <div className="p-4 text-xs text-secondary text-center">
                {t('history.conflict.noFieldDiffs')}
              </div>
            ) : (
              differingFields.map((item) => {
                const choice = getChoice(item.field);
                const formattedMine = formatDiffValue(
                  item.myValue,
                  t('history.drawer.emptyValue'),
                  { t, locale, field: item.field }
                );
                const formattedTheirs = formatDiffValue(
                  item.theirValue,
                  t('history.drawer.emptyValue'),
                  { t, locale, field: item.field }
                );
                const mineLabel = t('history.conflict.colMine');
                const theirsLabel = t('history.conflict.colTheirs', {
                  editor: editorName,
                });

                if (!mergeMode) {
                  return (
                    <div
                      key={item.field}
                      data-testid={`conflict-field-${item.field}`}
                      className="grid grid-cols-1 sm:grid-cols-3 gap-2 p-3 items-center text-xs"
                    >
                      <div className="font-semibold text-primary">
                        {item.label}
                      </div>

                      <div className="min-w-0 rounded-xl border border-border bg-canvas/60 p-2.5 text-primary">
                        <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wide text-tertiary block mb-0.5">
                          {mineLabel}
                        </span>
                        <span className="break-words block">{formattedMine}</span>
                      </div>

                      <div className="min-w-0 rounded-xl border border-border bg-canvas/60 p-2.5 text-primary">
                        <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wide text-tertiary block mb-0.5">
                          {theirsLabel}
                        </span>
                        <span className="break-words block">{formattedTheirs}</span>
                      </div>
                    </div>
                  );
                }

                return (
                  <RadioChoiceRow
                    key={item.field}
                    data-testid={`conflict-field-${item.field}`}
                    rowLabel={item.label}
                    ariaLabel={item.label || item.field}
                    value={choice}
                    onChange={(nextChoice) =>
                      setSelectedSourceByField((prev) => ({
                        ...prev,
                        [item.field]: nextChoice,
                      }))
                    }
                    options={[
                      {
                        value: 'mine',
                        label: formattedMine,
                        mobileHeader: mineLabel,
                      },
                      {
                        value: 'theirs',
                        label: formattedTheirs,
                        mobileHeader: theirsLabel,
                      },
                    ]}
                  />
                );
              })
            )}
          </div>
        </div>
      </div>
    </Dialog>
  );
};
