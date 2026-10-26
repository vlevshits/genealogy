import React, { useState, useEffect, Suspense } from 'react';
import { Compass, Sparkles, Layers, Shield, FileText, GitFork } from 'lucide-react';
import { SystemStatusCard } from './components/SystemStatusCard';
import { AppShell, Button, Badge, Spinner } from './components/ui';
import { useI18n } from './i18n';

const DevUiWorkbench = React.lazy(() => import('./pages/DevUiWorkbench'));

export const App: React.FC = () => {
  const { t } = useI18n();
  const [path, setPath] = useState<string>(() =>
    typeof window !== 'undefined' ? window.location.pathname : '/'
  );

  useEffect(() => {
    const onPopState = () => setPath(window.location.pathname);
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, []);

  const navigate = (nextPath: string) => {
    if (typeof window !== 'undefined') {
      window.history.pushState({}, '', nextPath);
    }
    setPath(nextPath);
  };

  if (path.startsWith('/dev/ui')) {
    return (
      <AppShell
        currentPath={path}
        onNavigate={navigate}
        breadcrumbs={[
          { id: 'home', label: t('nav.home'), onClick: () => navigate('/') },
          { id: 'dev-ui', label: t('nav.devWorkbench') },
        ]}
      >
        <Suspense
          fallback={
            <div className="flex min-h-[50vh] items-center justify-center gap-2 text-sm text-secondary">
              <Spinner size="md" />
              <span>{t('common.loading')}</span>
            </div>
          }
        >
          <DevUiWorkbench />
        </Suspense>
      </AppShell>
    );
  }

  return (
    <AppShell currentPath={path} onNavigate={navigate}>
      <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Archival Hero Banner */}
        <section className="relative overflow-hidden rounded-3xl border border-border bg-gradient-to-b from-surface to-surface/80 p-8 sm:p-12 shadow-xs">
          <div className="max-w-2xl space-y-4">
            <div className="inline-flex items-center gap-2 rounded-full border border-border bg-surface-muted px-3 py-1 text-xs font-medium text-brand">
              <Sparkles className="h-3.5 w-3.5" />
              <span>{t('app.badge')}</span>
            </div>

            <h1 className="font-serif text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight text-primary leading-tight">
              {t('app.title')}
            </h1>

            <p className="text-base sm:text-lg text-secondary leading-relaxed">
              {t('app.subtitle')}. {t('app.heroDescription')}
            </p>

            <div className="pt-2">
              <Button
                variant="primary"
                size="lg"
                leftIcon={<GitFork className="h-4 w-4" />}
                onClick={() => navigate('/tree')}
              >
                {t('app.openWorkbench')}
              </Button>
            </div>
          </div>

          <div className="mt-8 flex flex-wrap gap-4 text-xs font-medium text-secondary">
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Shield className="h-4 w-4 text-success" />
              <span>{t('app.zeroCloud')}</span>
            </div>
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Compass className="h-4 w-4 text-brand" />
              <span>{t('app.bilingual')}</span>
            </div>
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Layers className="h-4 w-4 text-accent" />
              <span>{t('app.gedcomReady')}</span>
            </div>
          </div>
        </section>

        {/* Live System Status */}
        <section>
          <SystemStatusCard />
        </section>

        {/* Lineage & Gender Identity Tokens Showcase */}
        <section className="rounded-2xl border border-border bg-surface p-6 shadow-xs">
          <h3 className="text-sm font-semibold text-primary uppercase tracking-wider mb-2">
            {t('app.paletteTitle')}
          </h3>
          <p className="text-xs text-secondary mb-4">{t('app.paletteSubtitle')}</p>

          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-male shadow-2xs" />
              <div>
                <div className="font-semibold text-primary">{t('app.lineageMale')}</div>
                <div className="text-secondary text-[11px]">{t('app.lineageMaleDesc')}</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-female shadow-2xs" />
              <div>
                <div className="font-semibold text-primary">{t('app.lineageFemale')}</div>
                <div className="text-secondary text-[11px]">{t('app.lineageFemaleDesc')}</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-accent shadow-2xs" />
              <div>
                <div className="font-semibold text-primary">{t('app.lineageGold')}</div>
                <div className="text-secondary text-[11px]">{t('app.lineageGoldDesc')}</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-neutral shadow-2xs" />
              <div>
                <div className="font-semibold text-primary">{t('app.lineageNeutral')}</div>
                <div className="text-secondary text-[11px]">{t('app.lineageNeutralDesc')}</div>
              </div>
            </div>
          </div>
        </section>

        {/* Archive Overview */}
        <section className="rounded-2xl border border-border bg-surface p-6 shadow-xs">
          <div className="flex items-center gap-2 font-semibold text-primary mb-4">
            <FileText className="h-5 w-5 text-brand" />
            <span>{t('app.phasesTitle')}</span>
          </div>

          <div className="space-y-3 text-xs sm:text-sm">
            <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-surface-muted/50">
              <span className="font-medium text-primary">{t('app.phaseSkeleton')}</span>
              <Badge variant="success">{t('app.completed')}</Badge>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl border border-brand/40 bg-brand/5">
              <span className="font-semibold text-primary">{t('app.phase0')}</span>
              <Badge variant="brand">{t('app.completed')}</Badge>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-canvas">
              <span className="text-secondary">{t('app.phase1')}</span>
              <Badge variant="neutral">{t('app.planned')}</Badge>
            </div>
          </div>
        </section>
      </div>
    </AppShell>
  );
};

export default App;
