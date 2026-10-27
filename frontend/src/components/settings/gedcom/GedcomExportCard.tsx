import React, { useState } from 'react';
import {
  Download,
  FileArchive,
  FileCode2,
  FileDown,
} from 'lucide-react';
import {
  Button,
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  Divider,
  RadioCardGroup,
  Switch,
  useToast,
} from '../../ui';
import {
  exportGedcom,
  GedcomExportOptions,
  GedcomVersionOption,
  getGedcomExportUrl,
} from '../../../api/client';
import { useI18n } from '../../../i18n';

export interface GedcomExportCardProps {
  initialOptions?: Partial<GedcomExportOptions>;
  onExportTriggered?: (options: GedcomExportOptions, url: string) => void;
}

export const GedcomExportCard: React.FC<GedcomExportCardProps> = ({
  initialOptions,
  onExportTriggered,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [version, setVersion] = useState<GedcomVersionOption>(
    initialOptions?.version ?? '5.5.1'
  );
  const [includeMedia, setIncludeMedia] = useState<boolean>(
    initialOptions?.include_media ?? true
  );
  const [redactLiving, setRedactLiving] = useState<boolean>(
    initialOptions?.redact_living ?? true
  );
  const [includePrivateNotes, setIncludePrivateNotes] = useState<boolean>(
    initialOptions?.include_private_notes ?? false
  );
  const [exporting, setExporting] = useState<boolean>(false);

  const handleDownloadExport = async () => {
    if (exporting) return;
    setExporting(true);

    const options: GedcomExportOptions = {
      version,
      include_media: includeMedia,
      redact_living: redactLiving,
      include_private_notes: includePrivateNotes,
    };
    const exportUrl = getGedcomExportUrl(options);
    onExportTriggered?.(options, exportUrl);

    try {
      const { blob, filename } = await exportGedcom(options);
      if (typeof window !== 'undefined' && typeof document !== 'undefined') {
        const objectUrl = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = objectUrl;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(objectUrl);
      }
      addToast({
        type: 'success',
        title: t('gedcom.export.startedToast'),
        description: filename,
      });
    } catch {
      if (typeof document !== 'undefined') {
        const link = document.createElement('a');
        link.href = exportUrl;
        link.download = includeMedia
          ? `family-tree-${version}.zip`
          : `family-tree-${version}.ged`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      }
      addToast({
        type: 'success',
        title: t('gedcom.export.startedToast'),
      });
    } finally {
      setExporting(false);
    }
  };

  return (
    <Card data-testid="gedcom-export-card">
      <CardHeader className="space-y-1">
        <div className="flex items-center gap-2">
          <FileDown className="h-4 w-4 text-brand" />
          <h2 className="text-base font-semibold text-primary">
            {t('gedcom.export.title')}
          </h2>
        </div>
        <p className="text-xs text-secondary">
          {t('gedcom.export.subtitle')}
        </p>
      </CardHeader>

      <CardBody className="space-y-5">
        {/* Format Standard Selector */}
        <div className="space-y-2">
          <div className="text-xs font-semibold uppercase tracking-wider text-secondary">
            {t('gedcom.export.formatLabel')}
          </div>
          <RadioCardGroup<GedcomVersionOption>
            ariaLabel={t('gedcom.export.formatLabel')}
            value={version}
            onChange={setVersion}
            options={[
              {
                value: '5.5.1',
                label: t('gedcom.export.version551Label'),
                description: t('gedcom.export.version551Desc'),
                icon: <FileCode2 className="h-4 w-4" />,
              },
              {
                value: '7.0',
                label: t('gedcom.export.version70Label'),
                description: t('gedcom.export.version70Desc'),
                icon: <FileArchive className="h-4 w-4" />,
              },
            ]}
          />
        </div>

        <Divider />

        {/* Export Packaging & Privacy Options */}
        <div className="space-y-3.5">
          <Switch
            checked={includeMedia}
            onCheckedChange={setIncludeMedia}
            label={t('gedcom.export.includeMediaLabel')}
            description={t('gedcom.export.includeMediaDesc')}
          />

          <Switch
            checked={redactLiving}
            onCheckedChange={setRedactLiving}
            label={t('gedcom.export.redactLivingLabel')}
            description={t('gedcom.export.redactLivingDesc')}
          />

          <Switch
            checked={includePrivateNotes}
            onCheckedChange={setIncludePrivateNotes}
            label={t('gedcom.export.includePrivateNotesLabel')}
            description={t('gedcom.export.includePrivateNotesDesc')}
          />
        </div>
      </CardBody>

      <CardFooter className="flex items-center justify-end">
        <Button
          variant="primary"
          loading={exporting}
          leftIcon={<Download className="h-4 w-4" />}
          onClick={() => {
            void handleDownloadExport();
          }}
          data-testid="gedcom-export-download-btn"
        >
          {t('gedcom.export.downloadBtn')}
        </Button>
      </CardFooter>
    </Card>
  );
};
