import React, { useState } from 'react';
import { Download, KeyRound } from 'lucide-react';
import { Button, Checkbox, CopyButton } from '../ui';
import { useI18n } from '../../i18n';

export interface RecoveryCodesCardProps {
  codes: string[];
  requireConfirmation?: boolean;
  onContinue?: () => void;
}

export const RecoveryCodesCard: React.FC<RecoveryCodesCardProps> = ({
  codes,
  requireConfirmation = true,
  onContinue,
}) => {
  const { t } = useI18n();
  const [confirmed, setConfirmed] = useState(false);

  const handleDownloadTxt = () => {
    if (typeof document === 'undefined') return;
    const content = [
      'Family Tree Archive — Emergency Recovery Codes',
      '==============================================',
      ...codes.map((c, idx) => `${String(idx + 1).padStart(2, ' ')}. ${c}`),
      '',
    ].join('\n');

    const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'family-archive-recovery-codes.txt';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-5">
      <div className="space-y-1.5">
        <div className="flex items-center gap-2 text-sm font-semibold text-primary">
          <KeyRound className="h-4 w-4 text-brand shrink-0" />
          <span>{t('auth.recoveryCodes.title')}</span>
        </div>
        <p className="text-xs text-secondary leading-relaxed">
          {t('auth.recoveryCodes.subtitle')}
        </p>
      </div>

      <div
        aria-label={t('auth.recoveryCodes.title')}
        className="grid grid-cols-2 gap-2 rounded-xl border border-border bg-canvas p-4 font-mono text-xs sm:text-sm font-semibold text-primary"
      >
        {codes.map((code, idx) => (
          <div
            key={code}
            className="flex items-center gap-2 rounded-lg border border-border/70 bg-surface px-2.5 py-1.5"
          >
            <span className="text-[11px] text-tertiary select-none">
              {String(idx + 1).padStart(2, '0')}.
            </span>
            <span className="tracking-wider">{code}</span>
          </div>
        ))}
      </div>

      <div className="flex flex-wrap items-center justify-between gap-2">
        <CopyButton
          value={codes.join('\n')}
          label={t('auth.recoveryCodes.copyAllBtn')}
        />
        <Button
          variant="secondary"
          size="sm"
          leftIcon={<Download className="h-3.5 w-3.5" />}
          onClick={handleDownloadTxt}
        >
          {t('auth.recoveryCodes.downloadTxtBtn')}
        </Button>
      </div>

      {requireConfirmation && onContinue && (
        <div className="space-y-4 border-t border-border pt-4">
          <Checkbox
            checked={confirmed}
            onCheckedChange={setConfirmed}
            label={t('auth.recoveryCodes.confirmCheckboxLabel')}
          />
          <Button
            variant="primary"
            fullWidth
            disabled={!confirmed}
            onClick={onContinue}
          >
            {t('auth.recoveryCodes.continueBtn')}
          </Button>
        </div>
      )}
    </div>
  );
};
