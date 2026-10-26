import React, { createContext, useContext, useState } from 'react';
import { Dialog } from './dialog';
import { Button } from './button';
import { TypedConfirmInput } from './typed-confirm-input';
import { useI18n } from '../../i18n';

export interface ConfirmOptions {
  title: string;
  description: string;
  impactList?: string[];
  requirePhrase?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

export interface ConfirmDialogProps extends ConfirmOptions {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  open,
  onOpenChange,
  title,
  description,
  impactList,
  requirePhrase,
  confirmLabel,
  cancelLabel,
  destructive = true,
  onConfirm,
}) => {
  const { t } = useI18n();
  const [typedText, setTypedText] = useState('');

  const canConfirm = !requirePhrase || typedText === requirePhrase;

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) setTypedText('');
        onOpenChange(next);
      }}
      size="sm"
      title={title}
      description={description}
      footer={
        <>
          <Button
            variant="secondary"
            onClick={() => {
              setTypedText('');
              onOpenChange(false);
            }}
          >
            {cancelLabel || t('common.cancel')}
          </Button>
          <Button
            variant={destructive ? 'danger' : 'primary'}
            disabled={!canConfirm}
            onClick={() => {
              setTypedText('');
              onConfirm();
              onOpenChange(false);
            }}
          >
            {confirmLabel || (destructive ? t('common.delete') : t('common.confirm'))}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {impactList && impactList.length > 0 && (
          <div className="rounded-xl border border-danger/25 bg-danger/10 p-3 text-xs">
            <div className="font-semibold text-danger mb-1.5">{t('confirm.impactTitle')}</div>
            <ul className="list-disc pl-4 space-y-1 text-primary">
              {impactList.map((item, i) => (
                <li key={i}>{item}</li>
              ))}
            </ul>
          </div>
        )}
        {requirePhrase && (
          <TypedConfirmInput
            expectedPhrase={requirePhrase}
            value={typedText}
            onChange={(val) => setTypedText(val)}
          />
        )}
      </div>
    </Dialog>
  );
};

interface ConfirmContextValue {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextValue | null>(null);

export const ConfirmProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, setState] = useState<{
    options: ConfirmOptions;
    resolve: (val: boolean) => void;
  } | null>(null);

  const confirm = (options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      setState({ options, resolve });
    });
  };

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      {state && (
        <ConfirmDialog
          open={true}
          {...state.options}
          onOpenChange={(open) => {
            if (!open) {
              state.resolve(false);
              setState(null);
            }
          }}
          onConfirm={() => {
            state.resolve(true);
            setState(null);
          }}
        />
      )}
    </ConfirmContext.Provider>
  );
};

export function useConfirm() {
  const ctx = useContext(ConfirmContext);
  if (!ctx) {
    throw new Error('useConfirm must be used within ConfirmProvider');
  }
  return ctx.confirm;
}
