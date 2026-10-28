import React, { useEffect, useRef, useState } from 'react';
import { Archive, Check, CheckCircle2, Upload } from 'lucide-react';
import {
  createMediaAsset,
  MediaAssetSummaryDto,
  MediaBundleReconciliationReport,
  uploadMediaBundle,
  UploadProgress,
} from '../../api/client';
import { formatBytes } from '../settings/SystemDiagnosticsCard';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import {
  Alert,
  Button,
  Dialog,
  FileDropzone,
  ProgressBar,
  Spinner,
  Switch,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import { useI18n } from '../../i18n';

export interface MediaUploadDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultLinkedPersonId?: string;
  onUploaded?: (created: MediaAssetSummaryDto[]) => void;
  initialArchiveReport?: MediaBundleReconciliationReport | null;
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve) => {
    if (typeof FileReader === 'undefined') {
      resolve('');
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      resolve(typeof reader.result === 'string' ? reader.result : '');
    };
    reader.onerror = () => resolve('');
    reader.readAsDataURL(file);
  });
}

export const MediaUploadDialog: React.FC<MediaUploadDialogProps> = ({
  open,
  onOpenChange,
  defaultLinkedPersonId,
  onUploaded,
  initialArchiveReport = null,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [defaultTitle, setDefaultTitle] = useState<string>('');
  const [dateTaken, setDateTaken] = useState<string>('');
  const [placePath, setPlacePath] = useState<string>('');
  const [isPrivate, setIsPrivate] = useState<boolean>(false);
  const [uploading, setUploading] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);
  const [archiveStage, setArchiveStage] = useState<
    'uploading' | 'matching' | 'thumbnails' | 'done' | null
  >(null);
  const [archiveReport, setArchiveReport] =
    useState<MediaBundleReconciliationReport | null>(initialArchiveReport);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const archiveDoneRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (archiveReport) {
      archiveDoneRef.current?.focus();
    }
  }, [archiveReport]);

  const isArchive = selectedFiles.some((f) => /\.(zip|gdz)$/i.test(f.name));

  const handleReset = () => {
    setSelectedFiles([]);
    setDefaultTitle('');
    setDateTaken('');
    setPlacePath('');
    setIsPrivate(false);
    setUploading(false);
    setProgress(0);
    setUploadProgress(null);
    setArchiveStage(null);
    setArchiveReport(null);
    setErrorMsg(null);
  };

  const handleUploadArchive = async (file: File) => {
    setUploading(true);
    setArchiveStage('uploading');
    setProgress(0);
    setUploadProgress({ loaded: 0, total: file.size, percent: 0 });
    setErrorMsg(null);

    try {
      const report = await uploadMediaBundle(file, {
        ingest_unmatched: true,
        onProgress: (p) => {
          setUploadProgress(p);
          setProgress(p.percent);
          if (p.percent >= 100) {
            setArchiveStage('matching');
          }
        },
      });

      setArchiveStage('thumbnails');
      // Brief visual state before finalizing
      setArchiveReport(report);
      setArchiveStage('done');

      addToast({
        type: 'success',
        title: t('mediaArchive.archiveSummaryTitle'),
      });
    } catch (err) {
      setErrorMsg(getLocalizedErrorMessage(err, t));
      setArchiveStage(null);
    } finally {
      setUploading(false);
    }
  };

  const handleUploadBatch = async (filesToUpload: File[]) => {
    const batch = filesToUpload.length > 0 ? filesToUpload : selectedFiles;
    if (batch.length === 0 || uploading) return;

    if (isArchive) {
      const archiveFile = batch.find((f) => /\.(zip|gdz)$/i.test(f.name)) || batch[0];
      await handleUploadArchive(archiveFile);
      return;
    }

    setUploading(true);
    setProgress(10);
    setErrorMsg(null);

    const createdAssets: MediaAssetSummaryDto[] = [];
    try {
      for (let i = 0; i < batch.length; i += 1) {
        const file = batch[i];
        const dataUrl = await readFileAsDataUrl(file);
        const titleForAsset =
          batch.length === 1 && defaultTitle.trim()
            ? defaultTitle.trim()
            : file.name.replace(/\.[^.]+$/, '');

        const created = await createMediaAsset({
          file_name: file.name,
          mime_type: file.type || 'image/webp',
          data: dataUrl || undefined,
          title: titleForAsset,
          date_taken: dateTaken.trim() || null,
          place_path: placePath.trim() || null,
          is_private: isPrivate,
          linked_person_ids: defaultLinkedPersonId
            ? [defaultLinkedPersonId]
            : undefined,
        });
        createdAssets.push(created);
        setProgress(Math.round(((i + 1) / batch.length) * 100));
      }

      addToast({
        type: 'success',
        title: t('mediaArchive.uploadSuccessToast', {
          count: createdAssets.length,
        }),
      });
      onUploaded?.(createdAssets);
      handleReset();
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(getLocalizedErrorMessage(err, t));
    } finally {
      setUploading(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) handleReset();
        onOpenChange(nextOpen);
      }}
      size="xl"
      title={t('mediaArchive.uploadDialogTitle')}
      description={t('mediaArchive.uploadDialogSubtitle')}
    >
      <div className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        {/* Batch Ingestion Result Summary */}
        {archiveReport ? (
          <div
            className="space-y-4 rounded-xl border border-success/30 bg-success/5 p-5"
            data-testid="archive-ingestion-summary"
          >
            <div className="flex items-center gap-2 text-success font-semibold text-sm">
              <CheckCircle2 className="h-5 w-5 shrink-0" />
              <span>{t('mediaArchive.archiveSummaryTitle')}</span>
            </div>
            <p className="text-xs text-secondary leading-relaxed">
              {t('mediaArchive.archiveSummarySubtitle')}
            </p>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-1">
              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('mediaArchive.archiveFilesProcessed')}
                </div>
                <div className="text-lg font-bold text-primary font-mono mt-1">
                  {archiveReport.total_archive_files}
                </div>
              </div>
              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('mediaArchive.archiveReconciledStubs')}
                </div>
                <div className="text-lg font-bold text-success font-mono mt-1">
                  {archiveReport.reconciled_stubs}
                </div>
              </div>
              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('mediaArchive.archiveNewlyCreated')}
                </div>
                <div className="text-lg font-bold text-brand font-mono mt-1">
                  {archiveReport.newly_created}
                </div>
              </div>
              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('mediaArchive.archiveUnmatched')}
                </div>
                <div className="text-lg font-bold text-secondary font-mono mt-1">
                  {archiveReport.unmatched_files}
                </div>
              </div>
            </div>

            <div className="flex items-center justify-end pt-3 border-t border-border">
              <Button
                ref={archiveDoneRef}
                variant="primary"
                onClick={() => {
                  onUploaded?.([]);
                  handleReset();
                  onOpenChange(false);
                }}
                leftIcon={<Check className="h-4 w-4" />}
              >
                {t('mediaArchive.archiveDoneBtn')}
              </Button>
            </div>
          </div>
        ) : (
          <>
            <FileDropzone
              accept=".jpg,.jpeg,.png,.webp,.gif,.tif,.tiff,.pdf,.mp3,.wav,.zip,.gdz"
              maxSizeMb={500}
              maxFiles={50}
              multiple
              onUpload={(files) => {
                setSelectedFiles(files);
              }}
            />

            {isArchive && (
              <Alert
                variant="info"
                icon={<Archive className="h-4 w-4 text-brand shrink-0 mt-0.5" />}
              >
                {t('mediaArchive.archiveNotice')}
              </Alert>
            )}

            {!isArchive && (
              <>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                  <FormField
                    name="default_title"
                    label={t('mediaArchive.fieldTitle')}
                  >
                    <TextInput
                      value={defaultTitle}
                      onChange={(e) => setDefaultTitle(e.target.value)}
                      placeholder={t('mediaArchive.fieldTitlePlaceholder')}
                    />
                  </FormField>

                  <FormField
                    name="date_taken"
                    label={t('mediaArchive.fieldDateTaken')}
                  >
                    <TextInput
                      value={dateTaken}
                      onChange={(e) => setDateTaken(e.target.value)}
                      placeholder={t('mediaArchive.fieldDatePlaceholder')}
                    />
                  </FormField>

                  <FormField
                    name="place_path"
                    label={t('mediaArchive.fieldPlace')}
                  >
                    <TextInput
                      value={placePath}
                      onChange={(e) => setPlacePath(e.target.value)}
                      placeholder={t('mediaArchive.fieldPlacePlaceholder')}
                    />
                  </FormField>
                </div>

                <Switch
                  checked={isPrivate}
                  onCheckedChange={setIsPrivate}
                  label={t('mediaArchive.privateToggleLabel')}
                />
              </>
            )}

            {/* Normal batch uploading progress */}
            {uploading && !isArchive && (
              <div role="status" aria-live="polite">
                <ProgressBar
                  value={progress}
                  label={t('mediaArchive.uploadingProgress')}
                  aria-label={t('mediaArchive.uploadingProgress')}
                />
              </div>
            )}

            {/* Archive multi-stage progress */}
            {uploading && isArchive && (
              <div
                role="status"
                aria-live="polite"
                className="space-y-3 rounded-xl border border-border bg-surface-muted/30 p-4"
              >
                <div className="flex items-center justify-between text-xs font-medium">
                  <span className="text-primary font-semibold">
                    {archiveStage === 'uploading' && t('mediaArchive.stage1Uploading')}
                    {archiveStage === 'matching' && t('mediaArchive.stage2Matching')}
                    {archiveStage === 'thumbnails' && t('mediaArchive.stage3Thumbnails')}
                  </span>
                  <span className="font-mono text-secondary">
                    {uploadProgress
                      ? t('mediaArchive.stageProgressBytes', {
                          loaded: formatBytes(uploadProgress.loaded),
                          total: formatBytes(uploadProgress.total),
                          percent: uploadProgress.percent,
                        })
                      : `${progress}%`}
                  </span>
                </div>

                <ProgressBar
                  value={progress}
                  showPercentage={false}
                  variant={archiveStage === 'done' ? 'success' : 'brand'}
                  indeterminate={archiveStage === 'matching' || archiveStage === 'thumbnails'}
                  label={
                    archiveStage === 'uploading'
                      ? t('mediaArchive.stage1Uploading')
                      : archiveStage === 'matching'
                      ? t('mediaArchive.stage2Matching')
                      : archiveStage === 'thumbnails'
                      ? t('mediaArchive.stage3Thumbnails')
                      : undefined
                  }
                  aria-label={
                    archiveStage === 'uploading'
                      ? t('mediaArchive.stage1Uploading')
                      : archiveStage === 'matching'
                      ? t('mediaArchive.stage2Matching')
                      : archiveStage === 'thumbnails'
                      ? t('mediaArchive.stage3Thumbnails')
                      : undefined
                  }
                  hideLabel
                />

                <div className="text-xs text-secondary">
                  {archiveStage === 'matching' && (
                    <div className="flex items-center gap-2">
                      <Spinner size="sm" />
                      <span>{t('mediaArchive.stageMatchingDetail')}</span>
                    </div>
                  )}
                  {archiveStage === 'thumbnails' && (
                    <div className="flex items-center gap-2">
                      <Spinner size="sm" />
                      <span>{t('mediaArchive.stageThumbnailsDetail')}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-border">
              <Button
                variant="secondary"
                onClick={() => {
                  handleReset();
                  onOpenChange(false);
                }}
              >
                {t('common.cancel')}
              </Button>
              <Button
                variant="primary"
                disabled={selectedFiles.length === 0 || uploading}
                loading={uploading}
                onClick={() => {
                  void handleUploadBatch(selectedFiles);
                }}
                leftIcon={<Upload className="h-4 w-4" />}
              >
                {isArchive
                  ? t('mediaArchive.submitArchiveBtn')
                  : t('mediaArchive.submitUploadBtn', {
                      count: selectedFiles.length,
                    })}
              </Button>
            </div>
          </>
        )}
      </div>
    </Dialog>
  );
};
