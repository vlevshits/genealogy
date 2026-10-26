import React from 'react';
import { Navbar } from './components/Navbar';
import { SystemStatusCard } from './components/SystemStatusCard';
import { useI18n } from './i18n';
import { Compass, Sparkles, Layers, Shield, FileText } from 'lucide-react';

export const App: React.FC = () => {
  const { t } = useI18n();

  return (
    <div className="min-h-screen flex flex-col bg-canvas text-primary transition-colors duration-200">
      <Navbar />

      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
        {/* Archival Hero Banner */}
        <section className="relative overflow-hidden rounded-3xl border border-border bg-gradient-to-b from-surface to-surface/80 p-8 sm:p-12 shadow-sm">
          <div className="max-w-2xl space-y-4">
            <div className="inline-flex items-center gap-2 rounded-full border border-border bg-surface-muted px-3 py-1 text-xs font-medium text-brand">
              <Sparkles className="h-3.5 w-3.5" />
              <span>Next-Gen Family Archive Platform</span>
            </div>

            <h1 className="font-serif text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight text-primary leading-tight">
              {t('app.title')}
            </h1>

            <p className="text-base sm:text-lg text-secondary leading-relaxed">
              {t('app.subtitle')}. Built with high-speed Rust Axum API backend, React 19 canvas visualizations, and single-instance Docker deployment.
            </p>
          </div>

          <div className="mt-8 flex flex-wrap gap-4 text-xs font-medium text-secondary">
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Shield className="h-4 w-4 text-success" />
              <span>Zero-Cloud Dependency</span>
            </div>
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Compass className="h-4 w-4 text-brand" />
              <span>Bilingual EN / RU Localization</span>
            </div>
            <div className="flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2">
              <Layers className="h-4 w-4 text-accent" />
              <span>GEDCOM 5.5.1 & 7.0 Compatible</span>
            </div>
          </div>
        </section>

        {/* Live System Architecture Status */}
        <section>
          <SystemStatusCard />
        </section>

        {/* Lineage & Gender Identity Tokens Showcase */}
        <section className="rounded-2xl border border-border bg-surface p-6 shadow-sm">
          <h3 className="text-sm font-semibold text-primary uppercase tracking-wider mb-2">
            Archival Lineage Token Palette
          </h3>
          <p className="text-xs text-secondary mb-4">
            Curated archival neutrals and lineage tones specified in design system tokens:
          </p>

          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-male shadow-xs" />
              <div>
                <div className="font-semibold text-primary">Paternal / Male</div>
                <div className="text-secondary text-[11px]">Dusty Indigo</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-female shadow-xs" />
              <div>
                <div className="font-semibold text-primary">Maternal / Female</div>
                <div className="text-secondary text-[11px]">Muted Dusty Rose</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-accent shadow-xs" />
              <div>
                <div className="font-semibold text-primary">Archival Gold</div>
                <div className="text-secondary text-[11px]">Milestones & Accents</div>
              </div>
            </div>

            <div className="flex items-center gap-2.5 rounded-xl border border-border p-3 bg-canvas">
              <span className="h-4 w-4 rounded-full bg-neutral shadow-xs" />
              <div>
                <div className="font-semibold text-primary">Unspecified</div>
                <div className="text-secondary text-[11px]">Neutral Slate</div>
              </div>
            </div>
          </div>
        </section>

        {/* Foundation & Roadmap Indicator */}
        <section className="rounded-2xl border border-border bg-surface p-6 shadow-sm">
          <div className="flex items-center gap-2 font-semibold text-primary mb-4">
            <FileText className="h-5 w-5 text-brand" />
            <span>Delivery Phases Overview</span>
          </div>

          <div className="space-y-3 text-xs sm:text-sm">
            <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-surface-muted/50">
              <span className="font-medium text-primary">Base Skeleton & Project Architecture</span>
              <span className="rounded-md bg-success/10 text-success font-semibold px-2.5 py-0.5 text-xs">
                Completed
              </span>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-canvas">
              <span className="text-secondary">Phase 0: Design System & Reusable Components (/dev/ui)</span>
              <span className="text-tertiary text-xs">Up Next</span>
            </div>
            <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-canvas">
              <span className="text-secondary">Phase 1: MVP Core (Auth, Data Model, Visualizer, Directory)</span>
              <span className="text-tertiary text-xs">Planned</span>
            </div>
          </div>
        </section>
      </main>

      <footer className="mt-auto border-t border-border bg-surface py-6 text-center text-xs text-secondary transition-colors duration-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-2">
          <span>Family Tree Archive &copy; 2026</span>
          <span className="font-mono text-[11px]">Axum + React 19 + Tailwind v4 + PostgreSQL 16</span>
        </div>
      </footer>
    </div>
  );
};

export default App;
