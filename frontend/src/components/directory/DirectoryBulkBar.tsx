import React from 'react';
import { Download, GitMerge, X } from 'lucide-react';
import { Badge, Button } from '../ui';
import { useI18n } from '../../i18n';

export interface DirectoryBulkBarProps {
  selectedIds: string[];
  canMerge: boolean;
  onCompareMerge: (idA: string, idB: string) => void;
  onExportGedcom: (ids: string[]) => void;
  onClearSelection: () => void;
}

export const DirectoryBulkBar: React.FC<DirectoryBulkBarProps> = ({
  selectedIds,
  canMerge,
  onCompareMerge,
  onExportGedcom,
  onClearSelection,
}) => {
  const { t } = useI18n();

  if (selectedIds.length === 0) return null;

  const isPairSelected = selectedIds.length === 2;

  return (
    <div
      role="region"
      aria-label={t('directory.bulkActionsAria')}
      className="fixed bottom-20 md:bottom-6 left-1/2 z-40 -translate-x-1/2 w-[calc(100%-2rem)] max-w-2xl rounded-2xl border border-border bg-surface/95 backdrop-blur-md px-4 py-3 shadow-2xl flex flex-wrap items-center justify-between gap-3"
    >
      <div className="flex items-center gap-2.5">
        <Badge variant="brand">
          {t('directory.selectedCount', { count: selectedIds.length })}
        </Badge>
        {!isPairSelected && canMerge && (
          <span className="hidden sm:inline text-xs text-secondary">
            {t('directory.mergeSelectTwoHint')}
          </span>
        )}
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {canMerge && (
          <Button
            variant="secondary"
            size="sm"
            disabled={!isPairSelected}
            onClick={() => {
              if (isPairSelected) {
                onCompareMerge(selectedIds[0], selectedIds[1]);
              }
            }}
            leftIcon={<GitMerge className="h-3.5 w-3.5" />}
          >
            {t('directory.compareMergeBtn')}
          </Button>
        )}

        <Button
          variant="secondary"
          size="sm"
          onClick={() => onExportGedcom(selectedIds)}
          leftIcon={<Download className="h-3.5 w-3.5" />}
        >
          {t('directory.exportSelectionGedcomBtn')}
        </Button>

        <Button
          variant="ghost"
          size="sm"
          onClick={onClearSelection}
          leftIcon={<X className="h-3.5 w-3.5" />}
        >
          {t('directory.deselectAllBtn')}
        </Button>
      </div>
    </div>
  );
};
