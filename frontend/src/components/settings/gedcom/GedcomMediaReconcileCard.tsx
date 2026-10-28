import React, { useEffect, useRef, useState } from 'react';
import {
  AlertTriangle,
  Archive,
  ArrowRight,
  CheckCircle2,
  RotateCw,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  FileDropzone,
  ProgressBar,
  Spinner,
  useToast,
} from '../../ui';
import {
  fetchMediaArchive,
  MediaBundleReconciliationReport,
  uploadGedcomMediaBundle,
  UploadProgress,
} from '../../../api/client';
import { formatBytes } from '../SystemDiagnosticsCard';
import { getLocalizedErrorMessage } from '../../../lib/error-formatter';
import { useI18n } from '../../../i18n';
import { cn } from '../../../lib/utils';

export interface MissingStubsSummary {
  total: number;
  samplePaths: string[];
}

export interface GedcomMediaReconcileCardProps {
  onNavigate?: (path: string) => void;
  initialReport?: MediaBundleReconciliationReport | null;
  initialMissingStubs?: MissingStubsSummary | null;
}

export const DEFAULT_RECONCILE_REPORT: MediaBundleReconciliationReport = {
  total_archive_files: 129,
  total_extracted: 129,
  reconciled_stubs: 129,
  newly_created: 0,
  unmatched_files: 0,
  unmatched_filenames: [],
  remaining_missing_stubs: 0,
  skipped_invalid_files: 0,
  skipped_files: 0,
  total_bytes_extracted: 53_982_412,
  duration_ms: 1420,
  errors: [],
};

export const GedcomMediaReconcileCard: React.FC<GedcomMediaReconcileCardProps> = ({
  onNavigate,
  initialReport = null,
  initialMissingStubs = null,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [missingStubs, setMissingStubs] = useState<MissingStubsSummary | null>(
    initialMissingStubs ?? null
  );
  const [loadingStubs, setLoadingStubs] = useState<boolean>(false);
  const [uploading, setUploading] = useState<boolean>(false);
  const [processing, setProcessing] = useState<boolean>(false);
  const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);
  const [report, setReport] = useState<MediaBundleReconciliationReport | null>(
    initialReport ?? null
  );
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const viewGalleryButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (report) {
      viewGalleryButtonRef.current?.focus();
    }
  }, [report]);

  const loadMissingStubs = async () => {
    try {
      setLoadingStubs(true);
      const res = await fetchMediaArchive({ is_missing: true, limit: 5 });
      const samplePaths = res.items
        .map((m) => m.original_file_name || m.file_name || '')
        .filter(Boolean);
      setMissingStubs({
        total: res.total,
        samplePaths,
      });
    } catch {
      // Ignore background fetch error
    } finally {
      setLoadingStubs(false);
    }
  };

  useEffect(() => {
    if (!initialMissingStubs) {
      void loadMissingStubs();
    }
  }, [initialMissingStubs]);

  const handleUpload = async (files: File[]) => {
    if (files.length === 0 || uploading) return;
    const file = files[0];

    setUploading(true);
    setProcessing(false);
    setErrorMessage(null);
    setUploadProgress({ loaded: 0, total: file.size, percent: 0 });

    try {
      const result = await uploadGedcomMediaBundle(file, {
        ingest_unmatched: false,
        onProgress: (p) => {
          setUploadProgress(p);
          if (p.percent >= 100) {
            setProcessing(true);
          }
        },
      });

      setReport(result);
      addToast({
        type: 'success',
        title: t('gedcom.reconcile.reportSuccess', {
          reconciled: result.reconciled_stubs,
          total: result.reconciled_stubs + result.remaining_missing_stubs,
        }),
      });

      void loadMissingStubs();
    } catch (err) {
      setErrorMessage(getLocalizedErrorMessage(err, t));
    } finally {
      setUploading(false);
      setProcessing(false);
      setUploadProgress(null);
    }
  };

  const handleReset = () => {
    setReport(null);
    setErrorMessage(null);
    void loadMissingStubs();
  };

  return (
    <Card data-testid="gedcom-media-reconcile-card">
      <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Archive className="h-5 w-5 text-brand" />
          <h2 className="text-base font-semibold text-primary">
            {t('gedcom.reconcile.title')}
          </h2>
        </div>
        <Badge variant="brand">
          {t('gedcom.reconcile.badgeFormat')}
        </Badge>
      </CardHeader>

      <CardBody className="space-y-4">
        <p className="text-sm text-secondary">
          {t('gedcom.reconcile.subtitle')}
        </p>

        {errorMessage && (
          <Alert variant="danger">
            {errorMessage}
          </Alert>
        )}

        {/* Missing stubs status banner */}
        {missingStubs && missingStubs.total > 0 && (
          <Alert
            variant="warning"
            icon={<AlertTriangle className="h-4 w-4 text-warning shrink-0 mt-0.5" />}
            title={t('gedcom.reconcile.missingStubsAlertTitle', {
              count: missingStubs.total,
            })}
          >
            {missingStubs.samplePaths.length > 0 && (
              <p className="font-mono text-xs opacity-90 mt-1 break-all">
                {t('gedcom.reconcile.samplePaths', {
                  paths:
                    missingStubs.samplePaths.join(', ') +
                    (missingStubs.total > missingStubs.samplePaths.length
                      ? ' …'
                      : ''),
                })}
              </p>
            )}
          </Alert>
        )}

        {missingStubs && missingStubs.total === 0 && !report && !loadingStubs && (
          <Alert
            variant="success"
            icon={<CheckCircle2 className="h-4 w-4 text-success shrink-0 mt-0.5" />}
            title={t('gedcom.reconcile.allReconciledTitle')}
          >
            {t('gedcom.reconcile.allReconciledSubtitle')}
          </Alert>
        )}

        {/* Uploading progress bar */}
        {uploading && (
          <div
            role="status"
            aria-live="polite"
            className="space-y-2 rounded-xl border border-border bg-surface-muted/30 p-4"
          >
            <div className="flex items-center justify-between text-xs font-medium">
              <span className="text-primary">
                {t('gedcom.reconcile.uploadingProgress')}
              </span>
              <span className="font-mono text-secondary">
                {uploadProgress
                  ? t('gedcom.reconcile.uploadingBytesProgress', {
                      loaded: formatBytes(uploadProgress.loaded),
                      total: formatBytes(uploadProgress.total),
                      percent: uploadProgress.percent,
                    })
                  : '0%'}
              </span>
            </div>
            <ProgressBar
              value={uploadProgress?.percent ?? 0}
              showPercentage={false}
              variant={uploadProgress?.percent === 100 ? 'success' : 'brand'}
              indeterminate={processing}
              label={t('gedcom.reconcile.uploadingProgress')}
              aria-label={t('gedcom.reconcile.uploadingProgress')}
              hideLabel
            />
            {processing && (
              <div className="flex items-center gap-2 pt-2 text-xs text-secondary animate-pulse">
                <Spinner size="sm" />
                <span>{t('gedcom.reconcile.extracting')}</span>
              </div>
            )}
          </div>
        )}

        {/* Dropzone */}
        {!report && !uploading && (
          <FileDropzone
            accept=".zip,.gdz"
            maxSizeMb={500}
            maxFiles={1}
            multiple={false}
            onUpload={handleUpload}
          />
        )}

        {/* Reconciliation Report Card */}
        {report && (
          <div
            className="space-y-4 rounded-xl border border-success/30 bg-success/5 p-5"
            data-testid="reconciliation-report-card"
          >
            <div className="flex items-center gap-2 text-success font-semibold text-sm">
              <CheckCircle2 className="h-5 w-5 shrink-0" />
              <span>
                {t('gedcom.reconcile.reportSuccess', {
                  reconciled: report.reconciled_stubs,
                  total: report.reconciled_stubs + report.remaining_missing_stubs,
                })}
              </span>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-1">
              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('gedcom.reconcile.statTotalArchive')}
                </div>
                <div className="text-lg font-bold text-primary font-mono mt-1">
                  {report.total_archive_files}
                </div>
              </div>

              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('gedcom.reconcile.statReconciled')}
                </div>
                <div className="text-lg font-bold text-success font-mono mt-1">
                  {report.reconciled_stubs}
                </div>
              </div>

              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('gedcom.reconcile.statUnmatched')}
                </div>
                <div className="text-lg font-bold text-secondary font-mono mt-1">
                  {report.unmatched_files}
                </div>
              </div>

              <div className="rounded-lg border border-border bg-surface p-3 text-center">
                <div className="text-xs text-secondary">
                  {t('gedcom.reconcile.statRemainingMissing')}
                </div>
                <div
                  className={cn(
                    'text-lg font-bold font-mono mt-1',
                    report.remaining_missing_stubs > 0
                      ? 'text-warning'
                      : 'text-primary'
                  )}
                >
                  {report.remaining_missing_stubs}
                </div>
              </div>
            </div>

            {report.unmatched_filenames.length > 0 && (
              <div className="text-xs text-secondary space-y-1">
                <div className="font-medium">
                  {t('gedcom.reconcile.unmatchedListTitle', {
                    count: report.unmatched_filenames.length,
                  })}
                </div>
                <div
                  tabIndex={0}
                  role="region"
                  aria-label={t('gedcom.reconcile.unmatchedListTitle', {
                    count: report.unmatched_filenames.length,
                  })}
                  className="font-mono text-[11px] max-h-24 overflow-y-auto bg-surface p-2 rounded border border-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                >
                  {report.unmatched_filenames.slice(0, 10).join(', ')}
                  {report.unmatched_filenames.length > 10 && ' …'}
                </div>
              </div>
            )}

            <div className="flex flex-wrap items-center justify-between gap-3 pt-3 border-t border-border">
              <Button
                variant="secondary"
                size="sm"
                onClick={handleReset}
                leftIcon={<RotateCw className="h-3.5 w-3.5" />}
              >
                {t('gedcom.reconcile.uploadAnother')}
              </Button>

              <Button
                ref={viewGalleryButtonRef}
                variant="primary"
                size="sm"
                onClick={() => {
                  if (onNavigate) {
                    onNavigate('/media');
                  } else if (typeof window !== 'undefined') {
                    window.location.href = '/media';
                  }
                }}
                rightIcon={<ArrowRight className="h-3.5 w-3.5" />}
              >
                {t('gedcom.reconcile.viewInGallery')}
              </Button>
            </div>
          </div>
        )}
      </CardBody>
    </Card>
  );
};
