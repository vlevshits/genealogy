import React, { useMemo, useState } from 'react';
import { AlertTriangle, Check, GitMerge, RefreshCw, Save } from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Dialog,
} from '../ui';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

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
  fieldLabels: Record<string, string> = {}
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
        label: fieldLabels[key] || key,
        myValue: mine,
        theirValue: theirs,
      });
    }
  }
  return list;
}

function stringifyFieldValue(value: unknown, emptyText: string): string {
  if (value === null || value === undefined || value === '') {
    return emptyText;
  }
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  }
  if (typeof value === 'object') {
    try {
      return JSON.stringify(value);
    } catch {
      return String(value);
    }
  }
  return String(value);
}

export const ConflictResolutionModal: React.FC<
  ConflictResolutionModalProps
> = ({
  open,
  onOpenChange,
  updatedByName,
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
  const { t } = useI18n();
  const [mergeMode, setMergeMode] = useState<boolean>(false);
  const resolvedTheirs = useMemo(
    () => theirData ?? theirChanges ?? {},
    [theirData, theirChanges]
  );

  const differingFields = useMemo<ConflictFieldEntry[]>(
    () => computeConflictFields(myChanges, resolvedTheirs, fieldLabels),
    [myChanges, resolvedTheirs, fieldLabels]
  );

  const [selectedSourceByField, setSelectedSourceByField] = useState<
    Record<string, 'mine' | 'theirs'>
  >({});

  const getChoice = (field: string): 'mine' | 'theirs' =>
    selectedSourceByField[field] ?? 'mine';

  const handleApplyFieldMerge = () => {
    const merged: Record<string, unknown> = { ...myChanges };
    for (const item of differingFields) {
      const choice = getChoice(item.field);
      merged[item.field] =
        choice === 'mine' ? item.myValue : item.theirValue;
    }
    onMergeFields?.(merged, remoteVersion);
    onMerge?.(merged, remoteVersion);
  };

  const editorName = updatedByName || t('history.conflict.anotherEditor');

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('history.conflict.title')}
      description={t('history.conflict.subtitle', { editor: editorName })}
    >
      <div
        data-testid="conflict-resolution-modal"
        className="space-y-4"
      >
        <Alert variant="warning">
          <div className="flex items-center justify-between gap-2">
            <span>
              {t('history.conflict.bannerText', {
                name: editorName,
                editor: editorName,
                time: String(remoteVersion),
                version: remoteVersion,
              })}
            </span>
            <Badge variant="warning" size="sm">
              v{remoteVersion}
            </Badge>
          </div>
        </Alert>

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
                return (
                  <div
                    key={item.field}
                    role="radiogroup"
                    aria-label={item.label || item.field}
                    data-testid={`conflict-field-${item.field}`}
                    className="grid grid-cols-1 sm:grid-cols-3 gap-2 p-3 items-center text-xs"
                  >
                    <div className="font-mono font-semibold text-primary">
                      {item.label}
                    </div>

                    <div
                      role="radio"
                      aria-checked={choice === 'mine'}
                      tabIndex={0}
                      onClick={() =>
                        setSelectedSourceByField((prev) => ({
                          ...prev,
                          [item.field]: 'mine',
                        }))
                      }
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          e.preventDefault();
                          setSelectedSourceByField((prev) => ({
                            ...prev,
                            [item.field]: 'mine',
                          }));
                        }
                      }}
                      className={cn(
                        'flex items-center justify-between gap-2 rounded-xl border p-2.5 cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                        choice === 'mine'
                          ? 'border-brand bg-brand/10 text-primary font-medium'
                          : 'border-border bg-canvas text-secondary hover:border-brand/40'
                      )}
                    >
                      <div className="min-w-0">
                        <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wide text-tertiary block mb-0.5">
                          {t('history.conflict.colMine')}
                        </span>
                        <span className="break-words block">
                          {stringifyFieldValue(
                            item.myValue,
                            t('history.drawer.emptyValue')
                          )}
                        </span>
                      </div>
                      {choice === 'mine' && (
                        <Check className="h-3.5 w-3.5 text-brand shrink-0" />
                      )}
                    </div>

                    <div
                      role="radio"
                      aria-checked={choice === 'theirs'}
                      tabIndex={0}
                      onClick={() =>
                        setSelectedSourceByField((prev) => ({
                          ...prev,
                          [item.field]: 'theirs',
                        }))
                      }
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          e.preventDefault();
                          setSelectedSourceByField((prev) => ({
                            ...prev,
                            [item.field]: 'theirs',
                          }));
                        }
                      }}
                      className={cn(
                        'flex items-center justify-between gap-2 rounded-xl border p-2.5 cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
                        choice === 'theirs'
                          ? 'border-brand bg-brand/10 text-primary font-medium'
                          : 'border-border bg-canvas text-secondary hover:border-brand/40'
                      )}
                    >
                      <div className="min-w-0">
                        <span className="sm:hidden text-[10px] font-semibold uppercase tracking-wide text-tertiary block mb-0.5">
                          {t('history.conflict.colTheirs', { editor: editorName })}
                        </span>
                        <span className="break-words block">
                          {stringifyFieldValue(
                            item.theirValue,
                            t('history.drawer.emptyValue')
                          )}
                        </span>
                      </div>
                      {choice === 'theirs' && (
                        <Check className="h-3.5 w-3.5 text-brand shrink-0" />
                      )}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* 3 Resolution Paths */}
        <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-border">
          <Button
            variant="secondary"
            size="sm"
            leftIcon={<RefreshCw className="h-3.5 w-3.5" />}
            onClick={() => onUseTheirs(resolvedTheirs)}
          >
            {t('history.conflict.useTheirsBtn')}
          </Button>

          <div className="flex flex-wrap items-center gap-2">
            {!mergeMode ? (
              <Button
                variant="secondary"
                size="sm"
                leftIcon={<GitMerge className="h-3.5 w-3.5" />}
                onClick={() => setMergeMode(true)}
              >
                {t('history.conflict.mergeFieldByFieldBtn')}
              </Button>
            ) : (
              <Button
                variant="primary"
                size="sm"
                leftIcon={<GitMerge className="h-3.5 w-3.5" />}
                onClick={handleApplyFieldMerge}
              >
                {t('history.conflict.applyMergedBtn')}
              </Button>
            )}

            <Button
              variant="primary"
              size="sm"
              leftIcon={<Save className="h-3.5 w-3.5" />}
              onClick={() => onKeepMine(remoteVersion)}
            >
              {t('history.conflict.keepMineBtn')}
            </Button>
          </div>
        </div>
      </div>
    </Dialog>
  );
};
