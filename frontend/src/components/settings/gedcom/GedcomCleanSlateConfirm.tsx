import React, { useState } from 'react';
import { AlertOctagon, History, ShieldAlert } from 'lucide-react';
import {
  Alert,
  Button,
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  Dialog,
  TextInput,
} from '../../ui';
import { FormField } from '../../form';
import { useI18n } from '../../../i18n';

export interface GedcomCleanSlateConfirmProps {
  open?: boolean;
  asDialog?: boolean;
  fileName: string;
  loading?: boolean;
  initialConfirmText?: string;
  onOpenChange?: (open: boolean) => void;
  onConfirm: (confirmText: string) => void;
  onCancel: () => void;
}

export function isCleanSlateConfirmed(value: string): boolean {
  const normalized = value.trim().toUpperCase();
  return (
    normalized === 'CLEAN_SLATE' ||
    normalized === 'REPLACE' ||
    normalized === 'ЗАМЕНИТЬ'
  );
}

export const GedcomCleanSlateConfirm: React.FC<
  GedcomCleanSlateConfirmProps
> = ({
  open = true,
  asDialog = false,
  fileName,
  loading = false,
  initialConfirmText = '',
  onOpenChange,
  onConfirm,
  onCancel,
}) => {
    const { t } = useI18n();
    const [confirmText, setConfirmText] = useState<string>(initialConfirmText);
    const confirmed = isCleanSlateConfirmed(confirmText);

    const bodyContent = (
      <div className="space-y-4" data-testid="gedcom-clean-slate-confirm">
        <Alert
          variant="danger"
          title={t('gedcom.cleanSlate.warningTitle')}
        >
          {t('gedcom.cleanSlate.warningBody', { fileName })}
        </Alert>

        <div className="flex items-start gap-2.5 rounded-xl border border-border bg-surface-muted/50 p-3 text-xs text-secondary">
          <History className="h-4 w-4 text-brand shrink-0 mt-0.5" />
          <div>
            <span className="font-semibold text-primary">
              {t('gedcom.cleanSlate.snapshotNoteTitle')}
            </span>{' '}
            <span>{t('gedcom.cleanSlate.snapshotNoteBody')}</span>
          </div>
        </div>

        <FormField
          name="clean_slate_confirm"
          label={t('gedcom.cleanSlate.inputLabel')}
          hint={t('gedcom.cleanSlate.inputHelper')}
        >
          <TextInput
            value={confirmText}
            onChange={(e) => setConfirmText(e.target.value)}
            placeholder="CLEAN_SLATE"
            aria-label={t('gedcom.cleanSlate.inputLabel')}
            className="font-mono uppercase"
            data-testid="gedcom-clean-slate-input"
          />
        </FormField>
      </div>
    );

    if (asDialog) {
      return (
        <Dialog
          open={open}
          onOpenChange={(nextOpen) => {
            onOpenChange?.(nextOpen);
            if (!nextOpen) onCancel();
          }}
          size="md"
          title={t('gedcom.cleanSlate.title')}
          description={t('gedcom.cleanSlate.subtitle')}
          footer={
            <>
              <Button variant="secondary" onClick={onCancel} disabled={loading}>
                {t('common.cancel')}
              </Button>
              <Button
                variant="danger"
                disabled={!confirmed || loading}
                loading={loading}
                leftIcon={<AlertOctagon className="h-4 w-4" />}
                onClick={() => onConfirm(confirmText.trim().toUpperCase())}
                data-testid="gedcom-clean-slate-submit-btn"
              >
                {t('gedcom.cleanSlate.confirmBtn')}
              </Button>
            </>
          }
        >
          {bodyContent}
        </Dialog>
      );
    }

    return (
      <Card className="border-danger/40">
        <CardHeader className="flex items-center gap-2">
          <ShieldAlert className="h-4 w-4 text-danger" />
          <div>
            <h3 className="text-sm font-semibold text-primary">
              {t('gedcom.cleanSlate.title')}
            </h3>
            <p className="text-xs text-secondary">
              {t('gedcom.cleanSlate.subtitle')}
            </p>
          </div>
        </CardHeader>
        <CardBody>{bodyContent}</CardBody>
        <CardFooter className="flex items-center justify-end gap-2">
          <Button variant="secondary" size="sm" onClick={onCancel} disabled={loading}>
            {t('common.cancel')}
          </Button>
          <Button
            variant="danger"
            size="sm"
            disabled={!confirmed || loading}
            loading={loading}
            leftIcon={<AlertOctagon className="h-4 w-4" />}
            onClick={() => onConfirm(confirmText.trim().toUpperCase())}
            data-testid="gedcom-clean-slate-submit-btn"
          >
            {t('gedcom.cleanSlate.confirmBtn')}
          </Button>
        </CardFooter>
      </Card>
    );
  };
