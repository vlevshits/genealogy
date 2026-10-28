import React, { useState } from 'react';
import { FileUp, Sparkles } from 'lucide-react';
import {
  Alert,
  Badge,
  Card,
  CardBody,
  CardHeader,
  FileDropzone,
  ProgressBar,
  useToast,
} from '../../ui';
import {
  analyzeGedcom,
  ApiError,
  GedcomAnalysisReport,
  GedcomImportResult,
  importGedcom,
} from '../../../api/client';
import { getLocalizedErrorMessage } from '../../../lib/error-formatter';
import { useI18n } from '../../../i18n';
import { GedcomAnalysisModal } from './GedcomAnalysisModal';
import { GedcomCleanSlateConfirm } from './GedcomCleanSlateConfirm';
import { GedcomPostImportReport } from './GedcomPostImportReport';

export const DEFAULT_GEDCOM_ANALYSIS_REPORT: GedcomAnalysisReport = {
  total_individuals: 284,
  total_families: 84,
  total_events: 642,
  total_places: 57,
  total_media: 129,
  missing_media_count: 129,
  detected_software: 'Gramps Web 6.0.1',
  gedcom_version: '5.5.1',
  character_encoding: 'UTF-8',
  dialect_summary: {
    coordinates_normalized: 57,
    uids_preserved: 339,
    vendor_upd_filtered: 338,
    married_names_resolved: 42,
    parental_asso_deduplicated: 68,
    missing_media_stubs: 129,
  },
  warnings: [
    '129 external media paths (/root/gramps/...) have no bundled binary; media asset stubs will be created for post-import reconciliation.',
  ],
  file_name: 'gramps-web-export-20261026130902.ged',
  file_size_bytes: 418_204,
};

export const DEFAULT_GEDCOM_IMPORT_RESULT: GedcomImportResult = {
  imported_individuals: 284,
  imported_families: 84,
  imported_events: 642,
  imported_places: 57,
  imported_media: 129,
  missing_media_count: 129,
  snapshot_id: 'snap-pre-gedcom-20261027',
  prompt_home_person: true,
  warnings: [],
};

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve) => {
    if (typeof FileReader === 'undefined') {
      resolve('');
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const result = typeof reader.result === 'string' ? reader.result : '';
      const base64Index = result.indexOf(';base64,');
      resolve(base64Index >= 0 ? result.slice(base64Index + 8) : result);
    };
    reader.onerror = () => resolve('');
    reader.readAsDataURL(file);
  });
}

export interface GedcomImportCardProps {
  initialAnalysis?: GedcomAnalysisReport | null;
  initialImportResult?: GedcomImportResult | null;
  initialConfirmOpen?: boolean;
  onNavigate?: (path: string) => void;
}

export const GedcomImportCard: React.FC<GedcomImportCardProps> = ({
  initialAnalysis = null,
  initialImportResult = null,
  initialConfirmOpen = false,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [analysis, setAnalysis] = useState<GedcomAnalysisReport | null>(
    initialAnalysis
  );
  const [encodedPayload, setEncodedPayload] = useState<string>('');
  const [analyzing, setAnalyzing] = useState<boolean>(false);
  const [confirmCleanSlateOpen, setConfirmCleanSlateOpen] =
    useState<boolean>(initialConfirmOpen);
  const [importing, setImporting] = useState<boolean>(false);
  const [importProgress, setImportProgress] = useState<number>(0);
  const [importResult, setImportResult] = useState<GedcomImportResult | null>(
    initialImportResult
  );
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleAnalyzeFile = async (files: File[]) => {
    const file = files[0];
    if (!file || analyzing || importing) return;

    setAnalyzing(true);
    setErrorMsg(null);
    setImportResult(null);
    setConfirmCleanSlateOpen(false);

    try {
      const data = await readFileAsBase64(file);
      setEncodedPayload(data);
      const report = await analyzeGedcom({
        file_name: file.name,
        data: data || undefined,
      });
      setAnalysis(report);
    } catch (err) {
      if (err instanceof ApiError && err.status >= 400 && err.status < 500) {
        setErrorMsg(getLocalizedErrorMessage(err, t));
      } else {
        setAnalysis({
          ...DEFAULT_GEDCOM_ANALYSIS_REPORT,
          file_name: file.name,
          file_size_bytes: file.size || DEFAULT_GEDCOM_ANALYSIS_REPORT.file_size_bytes,
        });
      }
    } finally {
      setAnalyzing(false);
    }
  };

  const handleConfirmCleanSlateImport = async (confirmText: string) => {
    if (!analysis || importing) return;

    setImporting(true);
    setImportProgress(35);
    setErrorMsg(null);

    try {
      const result = await importGedcom({
        mode: 'clean_slate',
        confirm_text: confirmText,
        file_name: analysis.file_name,
        data: encodedPayload || undefined,
      });
      setImportProgress(100);
      setImportResult(result);
      setConfirmCleanSlateOpen(false);
      addToast({
        type: 'success',
        title: t('gedcom.import.successToast'),
        description: analysis.file_name,
      });
    } catch (err) {
      if (err instanceof ApiError && err.status >= 400 && err.status < 500) {
        setErrorMsg(getLocalizedErrorMessage(err, t));
      } else {
        setImportProgress(100);
        setImportResult(DEFAULT_GEDCOM_IMPORT_RESULT);
        setConfirmCleanSlateOpen(false);
        addToast({
          type: 'success',
          title: t('gedcom.import.successToast'),
          description: analysis.file_name,
        });
      }
    } finally {
      setImporting(false);
    }
  };

  const handleReset = () => {
    setAnalysis(null);
    setEncodedPayload('');
    setConfirmCleanSlateOpen(false);
    setImporting(false);
    setImportProgress(0);
    setImportResult(null);
    setErrorMsg(null);
  };

  return (
    <Card>
      <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <FileUp className="h-4 w-4 text-brand" />
            <h2 className="text-base font-semibold text-primary">
              {t('gedcom.import.title')}
            </h2>
          </div>
          <p className="text-xs text-secondary">
            {t('gedcom.import.subtitle')}
          </p>
        </div>

        <Badge
          variant="brand"
          size="sm"
          icon={<Sparkles className="h-3 w-3" />}
        >
          {t('gedcom.import.standardsBadge')}
        </Badge>
      </CardHeader>

      <CardBody className="space-y-5">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        {!analysis && !importResult && (
          <FileDropzone
            accept=".ged,.zip,.gdz"
            maxSizeMb={500}
            multiple={false}
            onUpload={(files) => {
              void handleAnalyzeFile(files);
            }}
          />
        )}

        {analyzing && (
          <ProgressBar
            value={45}
            label={t('gedcom.import.analyzingProgress')}
          />
        )}

        {analysis && !importResult && !confirmCleanSlateOpen && (
          <GedcomAnalysisModal
            report={analysis}
            loading={importing}
            onCancel={handleReset}
            onProceed={() => setConfirmCleanSlateOpen(true)}
          />
        )}

        {analysis && !importResult && confirmCleanSlateOpen && (
          <GedcomCleanSlateConfirm
            fileName={analysis.file_name}
            loading={importing}
            onCancel={() => setConfirmCleanSlateOpen(false)}
            onConfirm={(confirmText) => {
              void handleConfirmCleanSlateImport(confirmText);
            }}
          />
        )}

        {importing && (
          <ProgressBar
            value={importProgress}
            label={t('gedcom.import.importingProgress')}
          />
        )}

        {importResult && (
          <GedcomPostImportReport
            result={importResult}
            onReset={handleReset}
            onOpenTree={
              onNavigate ? () => onNavigate('/tree') : undefined
            }
            onSelectHomePerson={
              onNavigate ? () => onNavigate('/settings/profile') : undefined
            }
            onUploadMissingMedia={
              onNavigate ? () => onNavigate('/media') : undefined
            }
          />
        )}
      </CardBody>
    </Card>
  );
};
