import React, { useState } from 'react';
import {
  Database,
  Download,
  HardDrive,
  Terminal,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  CopyButton,
  DescriptionList,
  Divider,
  EmptyState,
  Switch,
  useToast,
} from '../ui';
import {
  BackupArchiveInfo,
  BackupListResponse,
  getBackupDownloadUrl,
  triggerBackup,
} from '../../api/client';
import { useI18n } from '../../i18n';
import { formatBytes } from './SystemDiagnosticsCard';

export interface BackupsSettingsPanelProps {
  backupList: BackupListResponse;
  onBackupCreated?: (archive: BackupArchiveInfo) => void;
}

export const BackupsSettingsPanel: React.FC<BackupsSettingsPanelProps> = ({
  backupList,
  onBackupCreated,
}) => {
  const { t, locale } = useI18n();
  const { addToast } = useToast();

  const [includeGedcom, setIncludeGedcom] = useState<boolean>(true);
  const [creatingBackup, setCreatingBackup] = useState<boolean>(false);

  const formatTriggerLabel = (trigger: string): string => {
    switch (trigger) {
      case 'scheduled':
        return t('settings.backups.triggerScheduled');
      case 'pre_upgrade':
        return t('settings.backups.triggerPreUpgrade');
      case 'cli':
        return t('settings.backups.triggerCli');
      default:
        return t('settings.backups.triggerManual');
    }
  };

  const formatDate = (iso: string): string => {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  const handleCreateBackup = async () => {
    if (creatingBackup) return;
    setCreatingBackup(true);
    try {
      const res = await triggerBackup({ include_gedcom: includeGedcom });
      onBackupCreated?.(res.archive);
      addToast({
        type: 'success',
        title: t('settings.backups.createdToastTitle'),
        description: res.archive.filename,
      });

      if (typeof document !== 'undefined') {
        const link = document.createElement('a');
        link.href = getBackupDownloadUrl(res.archive.filename);
        link.download = res.archive.filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    } finally {
      setCreatingBackup(false);
    }
  };

  const latestArchiveName =
    backupList.items[0]?.filename ?? 'backup-2026-10-26.tar.gz';

  return (
    <div className="space-y-6">
      {/* Primary Full Archive Backup Card */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div className="space-y-1">
            <h2 className="text-base font-semibold text-primary">
              {t('settings.backups.title')}
            </h2>
            <p className="text-xs text-secondary">
              {t('settings.backups.subtitle')}
            </p>
          </div>
          <Button
            variant="primary"
            loading={creatingBackup}
            onClick={handleCreateBackup}
            leftIcon={<Download className="h-4 w-4" />}
          >
            {creatingBackup
              ? t('settings.backups.creatingCta')
              : t('settings.backups.downloadFullCta')}
          </Button>
        </CardHeader>

        <CardBody className="space-y-4">
          <Switch
            checked={includeGedcom}
            onCheckedChange={setIncludeGedcom}
            label={t('settings.backups.includeGedcomLabel')}
            description={t('settings.backups.includeGedcomDesc')}
          />

          <Divider />

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold uppercase tracking-wider text-tertiary">
                {t('settings.backups.scheduleTitle')}
              </span>
              <Badge
                variant={backupList.backup_enabled ? 'success' : 'neutral'}
              >
                {backupList.backup_enabled
                  ? t('settings.backups.scheduleEnabled')
                  : t('settings.backups.scheduleDisabled')}
              </Badge>
            </div>
            <DescriptionList
              items={[
                {
                  label: t('settings.backups.cronLabel'),
                  value: (
                    <span className="font-mono text-xs">
                      {backupList.cron_schedule}
                    </span>
                  ),
                },
                {
                  label: t('settings.backups.retentionLabel'),
                  value: t('settings.backups.retentionDays', {
                    days: backupList.retention_days,
                  }),
                },
              ]}
            />
          </div>
        </CardBody>
      </Card>

      {/* Available Backup Archives List */}
      <Card>
        <CardHeader className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <HardDrive className="h-4 w-4 text-brand" />
            <h3 className="text-sm font-semibold text-primary">
              {t('settings.backups.archivesTitle')}
            </h3>
          </div>
          <Badge variant="neutral">{backupList.items.length}</Badge>
        </CardHeader>

        <CardBody>
          {backupList.items.length === 0 ? (
            <EmptyState
              icon={<Database className="h-6 w-6" />}
              title={t('settings.backups.emptyTitle')}
              description={t('settings.backups.emptyDesc')}
            />
          ) : (
            <div className="divide-y divide-border">
              {backupList.items.map((item: BackupArchiveInfo) => (
                <div
                  key={item.filename}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 py-3.5 first:pt-0 last:pb-0"
                >
                  <div className="space-y-1 min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="font-mono text-xs sm:text-sm font-semibold text-primary truncate">
                        {item.filename}
                      </span>
                      <Badge variant="brand" size="sm">
                        {formatTriggerLabel(item.trigger)}
                      </Badge>
                      {item.includes_gedcom && (
                        <Badge variant="neutral" size="sm">
                          GEDZIP
                        </Badge>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-3 text-xs text-secondary">
                      <span>{formatDate(item.created_at)}</span>
                      <span>·</span>
                      <span className="font-mono">
                        {formatBytes(item.size_bytes)}
                      </span>
                      <span>·</span>
                      <span>
                        {t('settings.backups.colSchema')} v{item.schema_version}
                      </span>
                    </div>
                  </div>

                  <div className="shrink-0">
                    <Button
                      variant="secondary"
                      size="sm"
                      leftIcon={<Download className="h-3.5 w-3.5" />}
                      onClick={() => {
                        if (typeof window !== 'undefined') {
                          window.location.assign(
                            getBackupDownloadUrl(item.filename)
                          );
                        }
                      }}
                    >
                      {t('settings.backups.downloadBtn')}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>

      {/* Command-Line Disaster Recovery Reference (08 §4) */}
      <Card>
        <CardHeader className="flex items-center gap-2">
          <Terminal className="h-4 w-4 text-brand" />
          <div>
            <h3 className="text-sm font-semibold text-primary">
              {t('settings.backups.cliTitle')}
            </h3>
            <p className="text-xs text-secondary">
              {t('settings.backups.cliDesc')}
            </p>
          </div>
        </CardHeader>
        <CardBody className="space-y-3">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 rounded-xl border border-border bg-canvas p-3">
            <div className="space-y-0.5 min-w-0">
              <div className="text-[11px] font-medium text-secondary">
                {t('settings.backups.cliBackupLabel')}
              </div>
              <code className="block font-mono text-xs text-primary truncate">
                docker compose exec backend ./genealogy-cli backup
              </code>
            </div>
            <CopyButton value="docker compose exec backend ./genealogy-cli backup" />
          </div>

          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 rounded-xl border border-border bg-canvas p-3">
            <div className="space-y-0.5 min-w-0">
              <div className="text-[11px] font-medium text-secondary">
                {t('settings.backups.cliRestoreLabel')}
              </div>
              <code className="block font-mono text-xs text-primary truncate">
                {`docker compose exec backend ./genealogy-cli restore /app/data/backups/${latestArchiveName}`}
              </code>
            </div>
            <CopyButton
              value={`docker compose exec backend ./genealogy-cli restore /app/data/backups/${latestArchiveName}`}
            />
          </div>
        </CardBody>
      </Card>
    </div>
  );
};
