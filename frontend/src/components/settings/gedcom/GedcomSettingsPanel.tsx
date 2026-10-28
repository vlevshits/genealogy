import React from 'react';
import {
  GedcomAnalysisReport,
  GedcomExportOptions,
  GedcomImportResult,
  MediaBundleReconciliationReport,
} from '../../../api/client';
import { GedcomImportCard } from './GedcomImportCard';
import {
  GedcomMediaReconcileCard,
  MissingStubsSummary,
} from './GedcomMediaReconcileCard';
import { GedcomExportCard } from './GedcomExportCard';

export interface GedcomSettingsPanelProps {
  initialAnalysis?: GedcomAnalysisReport | null;
  initialImportResult?: GedcomImportResult | null;
  initialConfirmOpen?: boolean;
  initialExportOptions?: Partial<GedcomExportOptions>;
  initialReconcileReport?: MediaBundleReconciliationReport | null;
  initialMissingStubs?: MissingStubsSummary | null;
  onNavigate?: (path: string) => void;
  onExportTriggered?: (options: GedcomExportOptions, url: string) => void;
}

export const GedcomSettingsPanel: React.FC<GedcomSettingsPanelProps> = ({
  initialAnalysis = null,
  initialImportResult = null,
  initialConfirmOpen = false,
  initialExportOptions,
  initialReconcileReport = null,
  initialMissingStubs = null,
  onNavigate,
  onExportTriggered,
}) => {
  return (
    <div className="space-y-6" data-testid="gedcom-settings-panel">
      <GedcomImportCard
        initialAnalysis={initialAnalysis}
        initialImportResult={initialImportResult}
        initialConfirmOpen={initialConfirmOpen}
        onNavigate={onNavigate}
      />

      <GedcomMediaReconcileCard
        initialReport={initialReconcileReport}
        initialMissingStubs={initialMissingStubs}
        onNavigate={onNavigate}
      />

      <GedcomExportCard
        initialOptions={initialExportOptions}
        onExportTriggered={onExportTriggered}
      />
    </div>
  );
};

