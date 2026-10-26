import React, { useEffect, useState } from 'react';
import { Server, Database, ShieldCheck, Activity, Cpu } from 'lucide-react';
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
        setError(err.message || 'Failed to connect');
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  return (
    <div className="rounded-2xl border border-border bg-surface p-6 shadow-sm transition-colors duration-200">
      <div className="flex items-center justify-between pb-4 border-b border-border">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-muted text-brand">
            <Activity className="h-5 w-5" />
          </div>
          <div>
            <h2 className="font-semibold text-primary">{t('status.title')}</h2>
            <p className="text-xs text-secondary">
              Single-Instance Microservices Verification
            </p>
          </div>
        </div>
        <span className="rounded-full px-3 py-1 text-xs font-medium border border-border bg-surface-muted text-secondary">
          Phase 0: Skeleton Ready
        </span>
      </div>

      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        {/* Backend Status */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <Server className="h-5 w-5 text-brand mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">{t('status.backend')}</div>
            <div className="text-sm font-semibold text-primary">
              {loading ? t('common.loading') : health ? 'Axum 0.8 (Rust)' : 'Offline / Standalone'}
            </div>
            <div className="text-xs text-tertiary">
              {health?.version ? `v${health.version}` : error ? 'Check API' : 'Initializing'}
            </div>
          </div>
        </div>

        {/* Database Status */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <Database className="h-5 w-5 text-accent mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">{t('status.database')}</div>
            <div className="text-sm font-semibold text-primary">
              {loading ? t('common.loading') : health ? health.database : 'PostgreSQL 16'}
            </div>
            <div className="text-xs text-tertiary">
              {health?.database === 'connected' ? 'pg_trgm, unaccent, pgcrypto' : 'Ready for docker compose'}
            </div>
          </div>
        </div>

        {/* Security & Access */}
        <div className="flex items-start gap-3 rounded-xl border border-border bg-canvas p-4">
          <ShieldCheck className="h-5 w-5 text-success mt-0.5" />
          <div>
            <div className="text-xs font-medium text-secondary">Architecture</div>
            <div className="text-sm font-semibold text-primary">Private by Default</div>
            <div className="text-xs text-tertiary">1 Instance = 1 Family Tree</div>
          </div>
        </div>
      </div>

      {/* API documentation link */}
      <div className="mt-6 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-surface-muted p-3 text-xs text-secondary">
        <div className="flex items-center gap-2">
          <Cpu className="h-4 w-4 text-brand" />
          <span>Interactive OpenAPI (Swagger UI) explorer available at:</span>
        </div>
        <a
          href="/swagger-ui"
          target="_blank"
          rel="noopener noreferrer"
          className="font-mono font-medium text-brand hover:underline"
        >
          /swagger-ui
        </a>
      </div>
    </div>
  );
};
