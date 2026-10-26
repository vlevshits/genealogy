import React, { useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { Button, ButtonProps } from './button';
import { useI18n } from '../../i18n';

export interface CopyButtonProps extends Omit<ButtonProps, 'onClick'> {
  value: string;
  label?: string;
  copiedLabel?: string;
  onCopied?: () => void;
}

export const CopyButton: React.FC<CopyButtonProps> = ({
  value,
  label,
  copiedLabel,
  onCopied,
  variant = 'secondary',
  size = 'sm',
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(value);
      }
      setCopied(true);
      onCopied?.();
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <Button
      ref={ref}
      variant={variant}
      size={size}
      onClick={handleCopy}
      leftIcon={
        copied ? (
          <Check className="h-3.5 w-3.5 text-success" />
        ) : (
          <Copy className="h-3.5 w-3.5" />
        )
      }
      {...props}
    >
      {copied ? copiedLabel || t('common.copied') : label || t('common.copy')}
    </Button>
  );
};
