import React from 'react';
import {
  Disc,
  GitBranch,
  GitFork,
  Home,
  Maximize2,
  Minus,
  Network,
  Plus,
  Route,
  Workflow,
} from 'lucide-react';
import {
  Button,
  IconButton,
  SegmentedControl,
  Select,
} from '../ui';
import { TreeProjectionMode } from '../../api/client';
import { useI18n } from '../../i18n';

export interface TreeToolbarProps {
  mode: TreeProjectionMode;
  onModeChange: (mode: TreeProjectionMode) => void;
  generations: number;
  onGenerationsChange: (generations: number) => void;
  collateral?: boolean;
  onCollateralChange?: (collateral: boolean) => void;
  totalVisible?: number;
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onZoomReset: () => void;
  onCenterFocal: () => void;
  onFitToScreen: () => void;
  onOpenKinshipFinder?: () => void;
}

export const TreeToolbar: React.FC<TreeToolbarProps> = ({
  mode,
  onModeChange,
  generations,
  onGenerationsChange,
  collateral = true,
  onCollateralChange,
  totalVisible,
  zoom,
  onZoomIn,
  onZoomOut,
  onZoomReset,
  onCenterFocal,
  onFitToScreen,
  onOpenKinshipFinder,
}) => {
  const { t } = useI18n();

  const generationOptions = [
    { value: '3', label: t('tree.toolbar.generationsCount', { count: 3 }) },
    { value: '4', label: t('tree.toolbar.generationsCount', { count: 4 }) },
    { value: '5', label: t('tree.toolbar.generationsCount', { count: 5 }) },
    { value: '6', label: t('tree.toolbar.generationsCount', { count: 6 }) },
    { value: '10', label: t('tree.toolbar.generationsAll') },
  ];

  return (
    <>
      {/* Top Floating Bar: stacks vertically on narrow mobile (< 640px) so mode switcher & depth controls never collide */}
      <div className="pointer-events-none absolute top-3 inset-x-3 sm:top-4 sm:inset-x-6 z-20 flex flex-col sm:flex-row sm:flex-wrap lg:flex-nowrap sm:items-center sm:justify-between gap-2">
        {/* Projection Mode Switcher & Wide/Compact Selector */}
        <div className="flex flex-wrap items-center gap-2">
          <div
            onPointerDown={(e) => e.stopPropagation()}
            className="pointer-events-auto self-start max-w-full overflow-x-auto rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
          >
            <SegmentedControl<TreeProjectionMode>
              ariaLabel={t('tree.toolbar.modeSwitcherAria')}
              value={mode}
              onChange={onModeChange}
              size="md"
              options={[
                {
                  value: 'hourglass',
                  label: t('tree.modes.hourglass'),
                  icon: <GitFork className="h-3.5 w-3.5" />,
                },
                {
                  value: 'pedigree',
                  label: t('tree.modes.pedigree'),
                  icon: <GitBranch className="h-3.5 w-3.5" />,
                },
                {
                  value: 'descendant',
                  label: t('tree.modes.descendants'),
                  icon: <Workflow className="h-3.5 w-3.5" />,
                },
                {
                  value: 'fan',
                  label: t('tree.modes.fan'),
                  icon: <Disc className="h-3.5 w-3.5" />,
                },
                {
                  value: 'all',
                  label: t('tree.modes.all'),
                  icon: <Network className="h-3.5 w-3.5" />,
                },
              ]}
            />
          </div>

          {mode === 'hourglass' && onCollateralChange && (
            <div
              onPointerDown={(e) => e.stopPropagation()}
              data-testid="hourglass-collateral-toggle"
              title={t('tree.view_options.collateral_subtitle')}
              className="pointer-events-auto self-start max-w-full overflow-x-auto rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
            >
              <SegmentedControl<'wide' | 'compact'>
                ariaLabel={t('tree.view_options.collateral_toggle')}
                value={collateral ? 'wide' : 'compact'}
                onChange={(val) => onCollateralChange(val === 'wide')}
                size="md"
                options={[
                  {
                    value: 'wide',
                    label: t('tree.view_options.wide_hourglass'),
                  },
                  {
                    value: 'compact',
                    label: t('tree.view_options.compact_hourglass'),
                  },
                ]}
              />
            </div>
          )}
        </div>

        {/* Depth & Framing Controls */}
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="pointer-events-auto self-start sm:self-auto flex items-center gap-1.5 sm:gap-2 rounded-xl border border-border bg-surface/90 p-1.5 shadow-sm backdrop-blur-md"
        >
          {mode === 'all' ? (
            <div
              data-testid="tree-all-nodes-badge"
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border/60 bg-surface-muted/60 text-xs font-medium text-secondary"
            >
              <Network className="h-3.5 w-3.5 text-brand shrink-0" />
              <span>
                {t('tree.toolbar.allNodesCount', {
                  count: totalVisible ?? 0,
                })}
              </span>
            </div>
          ) : (
            <div className="w-36 sm:w-44">
              <Select
                ariaLabel={t('tree.toolbar.generationsAria')}
                value={String(generations)}
                onChange={(val) => onGenerationsChange(Number(val))}
                options={generationOptions}
              />
            </div>
          )}

          {onOpenKinshipFinder && (
            <Button
              size="sm"
              variant="secondary"
              data-testid="tree-open-kinship-btn"
              leftIcon={<Route className="h-3.5 w-3.5" />}
              onClick={onOpenKinshipFinder}
            >
              {t('tree.kinship.openBtn')}
            </Button>
          )}

          <IconButton
            label={t('tree.toolbar.centerHomeAria')}
            variant="secondary"
            onClick={onCenterFocal}
          >
            <Home className="h-4 w-4" />
          </IconButton>

          <IconButton
            label={t('tree.toolbar.fitScreenAria')}
            variant="secondary"
            onClick={onFitToScreen}
          >
            <Maximize2 className="h-4 w-4" />
          </IconButton>
        </div>
      </div>

      {/* Bottom-Left Floating Zoom Pill — lifted above BottomNav on mobile (< 768px) */}
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="absolute bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] md:bottom-6 left-3 sm:left-6 z-20 flex items-center gap-1 rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
      >
        <IconButton
          label={t('tree.toolbar.zoomOutAria')}
          variant="ghost"
          onClick={onZoomOut}
        >
          <Minus className="h-4 w-4" />
        </IconButton>

        <Button
          size="sm"
          variant="ghost"
          aria-label={t('tree.toolbar.zoomResetAria')}
          onClick={onZoomReset}
          className="min-w-14 min-h-11 font-mono text-xs font-semibold"
        >
          {Math.round(zoom * 100)}%
        </Button>

        <IconButton
          label={t('tree.toolbar.zoomInAria')}
          variant="ghost"
          onClick={onZoomIn}
        >
          <Plus className="h-4 w-4" />
        </IconButton>
      </div>
    </>
  );
};
