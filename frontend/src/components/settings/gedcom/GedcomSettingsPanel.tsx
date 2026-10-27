import React from 'react';
import {
  GedcomAnalysisReport,
  GedcomExportOptions,
  GedcomImportResult,
} from '../../../api/client';
import { GedcomImportCard } from './GedcomImportCard';
import { GedcomExportCard } from './GedcomExportCard';

export interface GedcomSettingsPanelProps {
  initialAnalysis?: GedcomAnalysisReport | null;
  initialImportResult?: GedcomImportResult | null;
  initialConfirmOpen?: boolean;
  initialExportOptions?: Partial<GedcomExportOptions>;
  onNavigate?: (path: string) => void;
  onExportTriggered?: (options: GedcomExportOptions, url: string) => void;
}

export const GedcomSettingsPanel: React.FC<GedcomSettingsPanelProps> = ({
  initialAnalysis = null,
  initialImportResult = null,
  initialConfirmOpen = false,
  initialExportOptions,
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

      <GedcomExportCard
        initialOptions={initialExportOptions}
        onExportTriggered={onExportTriggered}
      />
    </div>
  );
};
