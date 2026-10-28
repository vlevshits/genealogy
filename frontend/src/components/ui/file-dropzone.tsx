import React, { useRef, useState } from 'react';
import { UploadCloud, FileText, X, AlertCircle, CheckCircle2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Button } from './button';
import { IconButton } from './icon-button';
import { ProgressBar } from './progress-bar';
import { useI18n } from '../../i18n';

export interface QueuedFileItem {
  id: string;
  file: File;
  progress: number;
  error?: string;
}

export interface FileDropzoneProps {
  accept?: string;
  maxSizeMb?: number;
  maxFiles?: number;
  multiple?: boolean;
  onUpload?: (files: File[]) => void;
  className?: string;
}

export const FileDropzone: React.FC<FileDropzoneProps> = ({
  accept = '.jpg,.png,.webp,.pdf,.ged',
  maxSizeMb = 25,
  maxFiles = 10,
  multiple = true,
  onUpload,
  className,
}) => {
  const { t } = useI18n();
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [dragging, setDragging] = useState(false);
  const [queue, setQueue] = useState<QueuedFileItem[]>([]);

  const handleFiles = (fileList: FileList | null) => {
    if (!fileList) return;
    const incoming = Array.from(fileList);
    const validFiles: File[] = [];
    const nextItems: QueuedFileItem[] = [];

    const allowedExts = accept
      .split(',')
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean);

    incoming.forEach((f, idx) => {
      if (queue.length + nextItems.length >= maxFiles) {
        nextItems.push({
          id: `${f.name}-${Date.now()}-${idx}`,
          file: f,
          progress: 0,
          error: t('dropzone.errorTooMany', { maxFiles }),
        });
        return;
      }

      const sizeMb = f.size / (1024 * 1024);
      const ext = `.${f.name.split('.').pop()?.toLowerCase() ?? ''}`;
      const typeOk =
        allowedExts.length === 0 ||
        allowedExts.some((a) => a === ext || f.type.includes(a.replace('.', '')));

      if (!typeOk) {
        nextItems.push({
          id: `${f.name}-${Date.now()}-${idx}`,
          file: f,
          progress: 0,
          error: t('dropzone.errorInvalidType'),
        });
      } else if (sizeMb > maxSizeMb) {
        nextItems.push({
          id: `${f.name}-${Date.now()}-${idx}`,
          file: f,
          progress: 0,
          error: t('dropzone.errorTooLarge', { maxSize: maxSizeMb }),
        });
      } else {
        validFiles.push(f);
        nextItems.push({
          id: `${f.name}-${Date.now()}-${idx}`,
          file: f,
          progress: 100,
        });
      }
    });

    setQueue((prev) => [...prev, ...nextItems]);
    if (validFiles.length > 0) {
      onUpload?.(validFiles);
    }
  };

  return (
    <div className={cn('space-y-3', className)}>
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(e) => {
          e.preventDefault();
          setDragging(false);
          handleFiles(e.dataTransfer.files);
        }}
        onClick={() => inputRef.current?.click()}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            inputRef.current?.click();
          }
        }}
        aria-label={t('dropzone.title')}
        className={cn(
          'flex flex-col items-center justify-center rounded-2xl border-2 border-dashed p-6 text-center transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          dragging
            ? 'border-brand bg-brand/5'
            : 'border-border bg-surface hover:bg-surface-muted/50'
        )}
      >
        <input
          ref={inputRef}
          type="file"
          accept={accept}
          multiple={multiple}
          onChange={(e) => handleFiles(e.target.files)}
          className="sr-only"
          tabIndex={-1}
        />
        <UploadCloud className="mb-2 h-8 w-8 text-brand" />
        <p className="text-sm font-medium text-primary">{t('dropzone.title')}</p>
        <p className="mt-1 text-xs text-secondary">
          {t('dropzone.subtitle', { accept, maxSize: maxSizeMb })}
        </p>
        <Button variant="secondary" size="sm" className="mt-3 pointer-events-none">
          {t('dropzone.chooseFiles')}
        </Button>
      </div>

      {queue.length > 0 && (
        <ul className="space-y-2">
          {queue.map((item) => (
            <li
              key={item.id}
              className="flex items-center justify-between gap-3 rounded-xl border border-border bg-surface p-3 text-xs"
            >
              <div className="flex items-center gap-2.5 min-w-0 flex-1">
                <FileText className="h-4 w-4 shrink-0 text-brand" />
                <div className="min-w-0 flex-1 space-y-1">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-primary truncate">{item.file.name}</span>
                    <span className="font-mono text-tertiary shrink-0">
                      {(item.file.size / 1024).toFixed(1)} KB
                    </span>
                  </div>
                  {item.error ? (
                    <div className="flex items-center gap-1 text-danger">
                      <AlertCircle className="h-3.5 w-3.5 shrink-0" />
                      <span>{item.error}</span>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2">
                      <ProgressBar value={item.progress} showPercentage={false} variant="success" />
                      <CheckCircle2 className="h-3.5 w-3.5 text-success shrink-0" />
                    </div>
                  )}
                </div>
              </div>
              <IconButton
                size="sm"
                variant="ghost"
                label={t('dropzone.removeFile', { name: item.file.name })}
                onClick={() => setQueue((q) => q.filter((x) => x.id !== item.id))}
              >
                <X className="h-4 w-4" />
              </IconButton>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};
