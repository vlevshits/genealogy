import React from 'react';
import { cn } from '../../lib/utils';
import { Badge } from './badge';
import { useI18n } from '../../i18n';

export interface ComparisonFieldRow {
  fieldKey: string;
  label: string;
  leftValue: string;
  rightValue: string;
}

export interface ComparisonTableProps {
  rows: ComparisonFieldRow[];
  choices: Record<string, 'left' | 'right'>;
  onChoiceChange: (fieldKey: string, side: 'left' | 'right') => void;
  leftHeader?: string;
  rightHeader?: string;
  className?: string;
}

export const ComparisonTable: React.FC<ComparisonTableProps> = ({
  rows,
  choices,
  onChoiceChange,
  leftHeader,
  rightHeader,
  className,
}) => {
  const { t } = useI18n();

  return (
    <div className={cn('overflow-x-auto rounded-2xl border border-border bg-surface', className)}>
      <table className="w-full border-collapse text-left text-xs sm:text-sm">
        <thead>
          <tr className="border-b border-border bg-surface-muted/60 text-xs font-semibold text-secondary">
            <th className="px-4 py-3">{t('comparison.field')}</th>
            <th className="px-4 py-3">{t('comparison.status')}</th>
            <th className="px-4 py-3">{leftHeader || t('comparison.leftRecord')}</th>
            <th className="px-4 py-3">{rightHeader || t('comparison.rightRecord')}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-border">
          {rows.map((row) => {
            const isDiff = row.leftValue !== row.rightValue;
            const selectedSide = choices[row.fieldKey] ?? 'left';

            return (
              <tr
                key={row.fieldKey}
                className={cn(isDiff ? 'bg-warning/5' : 'bg-surface')}
              >
                <td className="px-4 py-3 font-medium text-primary">{row.label}</td>
                <td className="px-4 py-3">
                  <Badge variant={isDiff ? 'warning' : 'neutral'} size="sm">
                    {isDiff ? t('comparison.different') : t('comparison.identical')}
                  </Badge>
                </td>
                <td className="px-4 py-3">
                  <label className="flex items-center gap-2.5 cursor-pointer">
                    <input
                      type="radio"
                      name={`diff-${row.fieldKey}`}
                      checked={selectedSide === 'left'}
                      aria-label={t('comparison.chooseLeft', { field: row.label })}
                      onChange={() => onChoiceChange(row.fieldKey, 'left')}
                      className="accent-brand"
                    />
                    <span
                      className={cn(
                        selectedSide === 'left'
                          ? 'font-semibold text-primary'
                          : 'text-secondary'
                      )}
                    >
                      {row.leftValue || '—'}
                    </span>
                  </label>
                </td>
                <td className="px-4 py-3">
                  <label className="flex items-center gap-2.5 cursor-pointer">
                    <input
                      type="radio"
                      name={`diff-${row.fieldKey}`}
                      checked={selectedSide === 'right'}
                      aria-label={t('comparison.chooseRight', { field: row.label })}
                      onChange={() => onChoiceChange(row.fieldKey, 'right')}
                      className="accent-brand"
                    />
                    <span
                      className={cn(
                        selectedSide === 'right'
                          ? 'font-semibold text-primary'
                          : 'text-secondary'
                      )}
                    >
                      {row.rightValue || '—'}
                    </span>
                  </label>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
