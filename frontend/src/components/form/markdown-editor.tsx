import React, { useState } from 'react';
import { Bold, Italic, Link as LinkIcon, List } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Textarea } from '../ui/textarea';
import { MarkdownView } from '../ui/markdown-view';
import { IconButton } from '../ui/icon-button';
import { useI18n } from '../../i18n';

export interface MarkdownEditorProps {
  value?: string;
  defaultValue?: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  minRows?: number;
  className?: string;
}

export const MarkdownEditor: React.FC<MarkdownEditorProps> = ({
  value,
  defaultValue = '',
  onChange,
  placeholder,
  minRows = 5,
  className,
}) => {
  const { t } = useI18n();
  const [internalVal, setInternalVal] = useState(defaultValue);
  const [mode, setMode] = useState<'write' | 'preview'>('write');

  const current = value !== undefined ? value : internalVal;

  const update = (next: string) => {
    if (value === undefined) setInternalVal(next);
    onChange?.(next);
  };

  const insertToken = (snippet: string) => {
    update(`${current}${current ? ' ' : ''}${snippet}`);
  };

  return (
    <div className={cn('rounded-xl border border-border bg-surface overflow-hidden', className)}>
      <div className="flex items-center justify-between border-b border-border bg-surface-muted/50 px-3 py-1.5">
        <div role="tablist" className="flex items-center gap-1">
          <button
            type="button"
            role="tab"
            aria-selected={mode === 'write'}
            onClick={() => setMode('write')}
            className={cn(
              'rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
              mode === 'write'
                ? 'bg-surface text-primary shadow-2xs'
                : 'text-secondary hover:text-primary'
            )}
          >
            {t('markdown.write')}
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode === 'preview'}
            onClick={() => setMode('preview')}
            className={cn(
              'rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
              mode === 'preview'
                ? 'bg-surface text-primary shadow-2xs'
                : 'text-secondary hover:text-primary'
            )}
          >
            {t('markdown.preview')}
          </button>
        </div>

        {mode === 'write' && (
          <div className="flex items-center gap-0.5">
            <IconButton
              size="sm"
              label={t('markdown.bold')}
              onClick={() => insertToken('**bold**')}
            >
              <Bold className="h-3.5 w-3.5" />
            </IconButton>
            <IconButton
              size="sm"
              label={t('markdown.italic')}
              onClick={() => insertToken('*italic*')}
            >
              <Italic className="h-3.5 w-3.5" />
            </IconButton>
            <IconButton
              size="sm"
              label={t('markdown.link')}
              onClick={() => insertToken('[Archive](https://example.org)')}
            >
              <LinkIcon className="h-3.5 w-3.5" />
            </IconButton>
            <IconButton
              size="sm"
              label={t('markdown.list')}
              onClick={() => insertToken('\n- Item')}
            >
              <List className="h-3.5 w-3.5" />
            </IconButton>
          </div>
        )}
      </div>

      <div className="p-3">
        {mode === 'write' ? (
          <Textarea
            value={current}
            onChange={(e) => update(e.target.value)}
            placeholder={placeholder}
            minRows={minRows}
            className="border-none p-0 focus-visible:ring-0"
          />
        ) : (
          <div className="min-h-28 py-1">
            {current.trim() ? (
              <MarkdownView content={current} />
            ) : (
              <p className="text-xs italic text-tertiary">{t('markdown.emptyPreview')}</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
