import React from 'react';
import { Server } from 'lucide-react';
import {
  Badge,
  Card,
  CardBody,
  CardHeader,
  DescriptionList,
} from '../ui';
import { AdminSystemResponse } from '../../api/client';
import { useI18n } from '../../i18n';

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value >= 10 || unitIndex === 0 ? Math.round(value) : value.toFixed(1)} ${units[unitIndex]}`;
}

export interface SystemDiagnosticsCardProps {
  system: AdminSystemResponse;
}

export const SystemDiagnosticsCard: React.FC<SystemDiagnosticsCardProps> = ({
  system,
}) => {
  const { t, locale } = useI18n();

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

  return (
    <Card>
      <CardHeader className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2.5">
          <Server className="h-4.5 w-4.5 text-brand" />
          <div>
            <h2 className="text-sm font-semibold text-primary">
              {t('settings.system.title')}
            </h2>
            <p className="text-xs text-secondary">
              {t('settings.system.subtitle')}
            </p>
          </div>
        </div>
        <Badge variant="success">{t('settings.system.serverHealthy')}</Badge>
      </CardHeader>
      <CardBody>
        <DescriptionList
          items={[
            {
              label: t('settings.system.serverLabel'),
              value: t('settings.system.serverHealthy'),
            },
            {
              label: t('settings.system.databaseLabel'),
              value:
                system.database.status === 'connected' ||
                system.database.status === 'standalone_ready'
                  ? t('settings.system.databaseConnected', {
                      size: formatBytes(system.database.size_bytes),
                    })
                  : t('settings.system.databaseOffline'),
            },
            {
              label: t('settings.system.storageLabel'),
              value: t('settings.system.storageSummary', {
                used: formatBytes(system.storage.media_size_bytes),
                files: system.storage.media_files_count,
                free: formatBytes(system.storage.free_disk_bytes),
              }),
            },
            {
              label: t('settings.system.lastBackupLabel'),
              value: system.backups.last_backup_at
                ? formatDate(system.backups.last_backup_at)
                : t('settings.system.lastBackupNone'),
            },
            {
              label: t('settings.system.jobsLabel'),
              value: t('settings.system.jobsSummary', {
                active: system.jobs.active_workers,
                queued: system.jobs.queue_length,
              }),
            },
            {
              label: t('settings.system.versionLabel'),
              value: t('settings.system.versionValue', {
                version: system.app_version,
              }),
            },
          ]}
        />
      </CardBody>
    </Card>
  );
};
