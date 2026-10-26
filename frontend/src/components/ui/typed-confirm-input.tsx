import React, { useState, useId } from 'react';
import { TextInput } from './input';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface TypedConfirmInputProps {
  expectedPhrase: string;
  value?: string;
  onChange?: (value: string, matches: boolean) => void;
  className?: string;
}

export const TypedConfirmInput: React.FC<TypedConfirmInputProps> = ({
  expectedPhrase,
  value,
  onChange,
  className,
}) => {
  const { t } = useI18n();
  const inputId = useId();
  const [internalVal, setInternalVal] = useState('');
  const current = value !== undefined ? value : internalVal;

  return (
    <div className={cn('space-y-1.5', className)}>
      <label htmlFor={inputId} className="block text-xs font-medium text-secondary">
        {t('confirm.typePhraseLabel', { phrase: expectedPhrase })}
      </label>
      <TextInput
        id={inputId}
        value={current}
        placeholder={expectedPhrase}
        onChange={(e) => {
          const next = e.target.value;
          if (value === undefined) setInternalVal(next);
          onChange?.(next, next === expectedPhrase);
        }}
        className="font-mono"
      />
    </div>
  );
};
