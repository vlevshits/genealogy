import React, { useEffect, useState } from 'react';
import { Server, Database, ShieldCheck, Activity } from 'lucide-react';
import { fetchHealth, HealthResponse } from '../api/client';
import { useI18n } from '../i18n';

export const SystemStatusCard: React.FC = () => {
  const { t } = useI18n();
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchHealth()
      .then((data) => {
        setHealth(data);
        setError(null);
      })
      .catch((err) => {
        setError(err.message || t('status.degraded'));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [t]);

  return (
    <div className="rounded-2xl border border-border bg-surface p-6 shadow-xs transition-colors duration-200">
      <div className="flex items-center justify-between pb-4 border-b border-border">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-muted text-brand">
            <Activity className="h-5 w-5" />
          </div>
          <div>
            <h2 className="font-semibold text-primary">{t('status.title')}</h2>
            <p className="text-xs text-secondary">{t('status.subtitle')}</p>
          </div>
        </div>
        <span className="rounded-full px-3 py-1 text-xs font-medium border border-border bg-surface-muted text-secondary">
          {t('status.phaseBadge')}
        </span>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        {/* Server Status */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <Server className="h-5 w-5 text-brand mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">{t('status.backend')}</div>
            <div className="text-sm font-semibold text-primary">
              {loading
                ? t('common.loading')
                : health
                  ? t('status.healthy')
                  : t('status.degraded')}
            </div>
            <div className="text-xs text-tertiary">
              {health?.version
                ? t('status.version', { version: health.version })
                : error
                  ? t('status.degraded')
                  : t('status.connecting')}
            </div>
          </div>
        </div>

        {/* Database Status */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <Database className="h-5 w-5 text-accent mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">{t('status.database')}</div>
            <div className="text-sm font-semibold text-primary">
              {loading
                ? t('common.loading')
                : health?.database === 'connected'
                  ? t('status.connected')
                  : t('status.degraded')}
            </div>
            <div className="text-xs text-tertiary">{t('status.searchReady')}</div>
          </div>
        </div>

        {/* Privacy & Scope */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <ShieldCheck className="h-5 w-5 text-success mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">{t('status.architecture')}</div>
            <div className="text-sm font-semibold text-primary">{t('status.privateByDefault')}</div>
            <div className="text-xs text-tertiary">{t('status.singleInstance')}</div>
          </div>
        </div>
      </div>
    </div>
  );
};
