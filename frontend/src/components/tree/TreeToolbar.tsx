import React, { useState, useRef, useEffect } from 'react';
import {
  ArrowLeftRight,
  ArrowUpDown,
  Disc,
  GitBranch,
  GitFork,
  Home,
  Maximize2,
  Minus,
  Network,
  Plus,
  Route,
  SlidersHorizontal,
  Workflow,
  X,
} from 'lucide-react';
import {
  Button,
  IconButton,
  SegmentedControl,
  Select,
} from '../ui';
import { cn } from '../../lib/utils';
import { TreeProjectionMode } from '../../api/client';
import { TreeLayoutOrientation } from '../../lib/tree-layouts';
import { useI18n } from '../../i18n';

export interface TreeToolbarProps {
  mode: TreeProjectionMode;
  onModeChange: (mode: TreeProjectionMode) => void;
  generations: number;
  onGenerationsChange: (generations: number) => void;
  collateral?: boolean;
  onCollateralChange?: (collateral: boolean) => void;
  orientation?: TreeLayoutOrientation;
  onOrientationChange?: (orientation: TreeLayoutOrientation) => void;
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
  orientation = 'vertical',
  onOrientationChange,
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

  const modeOptions = [
    {
      value: 'hourglass' as TreeProjectionMode,
      label: t('tree.modes.hourglass'),
      icon: <GitFork className="h-3.5 w-3.5" />,
    },
    {
      value: 'pedigree' as TreeProjectionMode,
      label: t('tree.modes.pedigree'),
      icon: <GitBranch className="h-3.5 w-3.5" />,
    },
    {
      value: 'descendant' as TreeProjectionMode,
      label: t('tree.modes.descendants'),
      icon: <Workflow className="h-3.5 w-3.5" />,
    },
    {
      value: 'fan' as TreeProjectionMode,
      label: t('tree.modes.fan'),
      icon: <Disc className="h-3.5 w-3.5" />,
    },
    {
      value: 'all' as TreeProjectionMode,
      label: t('tree.modes.all'),
      icon: <Network className="h-3.5 w-3.5" />,
    },
  ];

  const [mobileOptionsOpen, setMobileOptionsOpen] = useState(false);
  const mobileOptionsRef = useRef<HTMLDivElement | null>(null);
  const optionsButtonRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    if (!mobileOptionsOpen) return;
    const onOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        mobileOptionsRef.current &&
        !mobileOptionsRef.current.contains(target) &&
        optionsButtonRef.current &&
        !optionsButtonRef.current.contains(target)
      ) {
        setMobileOptionsOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setMobileOptionsOpen(false);
    };
    document.addEventListener('mousedown', onOutside);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onOutside);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [mobileOptionsOpen]);

  const hasConfigurableOptions = mode !== 'all' && mode !== 'fan';
  const hasCustomOptions =
    hasConfigurableOptions &&
    (orientation === 'horizontal' ||
      (mode === 'hourglass' && !collateral) ||
      generations !== 4);

  return (
    <>
      {/* Top Floating Bar: Single unified bar on mobile (< 640px), split floating bar on desktop (>= 640px) */}
      <div className="pointer-events-none absolute top-3 inset-x-3 sm:top-4 sm:inset-x-6 z-20 flex flex-col sm:flex-row sm:flex-wrap lg:flex-nowrap sm:items-center sm:justify-between gap-2">
        {/* Mobile View (< 640px): Full-width streamlined toolbar with right-aligned icons */}
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="relative flex sm:hidden pointer-events-auto w-full items-center"
        >
          <div className="flex h-11 w-full items-center justify-between gap-1.5 rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md">
            {/* Mode Selector: uses all remaining width on the left */}
            <div className="flex-1 min-w-0">
              <Select<TreeProjectionMode>
                ariaLabel={t('tree.toolbar.modeSwitcherAria')}
                value={mode}
                onChange={onModeChange}
                size="sm"
                options={modeOptions}
              />
            </div>

            {/* Right-aligned action and option icons */}
            <div className="flex items-center gap-1 shrink-0">
              {/* Collapsible View Options Button (only for modes with configurable options) */}
              {hasConfigurableOptions && (
                <div className="relative">
                  <IconButton
                    ref={optionsButtonRef}
                    size="sm"
                    label={t('tree.toolbar.optionsBtn')}
                    variant={hasCustomOptions || mobileOptionsOpen ? 'secondary' : 'ghost'}
                    onClick={() => setMobileOptionsOpen(!mobileOptionsOpen)}
                    aria-expanded={mobileOptionsOpen}
                  >
                    <SlidersHorizontal
                      className={cn(
                        'h-4 w-4',
                        hasCustomOptions || mobileOptionsOpen
                          ? 'text-brand'
                          : 'text-secondary'
                      )}
                    />
                  </IconButton>
                  {hasCustomOptions && !mobileOptionsOpen && (
                    <span className="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full bg-brand ring-2 ring-surface pointer-events-none" />
                  )}
                </div>
              )}

              {/* Kinship Finder */}
              {onOpenKinshipFinder && (
                <IconButton
                  size="sm"
                  label={t('tree.kinship.openBtn')}
                  variant="ghost"
                  onClick={onOpenKinshipFinder}
                >
                  <Route className="h-4 w-4 text-secondary" />
                </IconButton>
              )}

              {/* Center Home */}
              <IconButton
                size="sm"
                label={t('tree.toolbar.centerHomeAria')}
                variant="ghost"
                onClick={onCenterFocal}
              >
                <Home className="h-4 w-4 text-secondary" />
              </IconButton>

              {/* Fit Screen */}
              <IconButton
                size="sm"
                label={t('tree.toolbar.fitScreenAria')}
                variant="ghost"
                onClick={onFitToScreen}
              >
                <Maximize2 className="h-4 w-4 text-secondary" />
              </IconButton>
            </div>
          </div>

          {/* Mobile Collapsible Options Panel (Drops down cleanly below toolbar, aligned right) */}
          <div
            ref={mobileOptionsRef}
            onPointerDown={(e) => e.stopPropagation()}
            className={cn(
              'absolute top-13 right-0 w-80 max-w-[calc(100vw-24px)] rounded-xl border border-border bg-surface/95 p-3.5 shadow-xl backdrop-blur-md z-30 flex flex-col gap-3 transition-all duration-150',
              !mobileOptionsOpen && 'hidden'
            )}
          >
            <div className="flex items-center justify-between border-b border-border/60 pb-2">
              <div className="flex items-center gap-1.5 text-xs font-semibold text-primary">
                <SlidersHorizontal className="h-3.5 w-3.5 text-brand" />
                <span>{t('tree.toolbar.viewSettings')}</span>
              </div>
              <IconButton
                label={t('common.close')}
                size="sm"
                variant="ghost"
                onClick={() => setMobileOptionsOpen(false)}
              >
                <X className="h-3.5 w-3.5" />
              </IconButton>
            </div>

            {/* Depth Selector */}
            {mode !== 'all' && (
              <div className="flex flex-col gap-1">
                <label className="text-xs font-medium text-secondary">
                  {t('tree.toolbar.generationsAria')}
                </label>
                <Select
                  ariaLabel={t('tree.toolbar.generationsAria')}
                  value={String(generations)}
                  onChange={(val) => onGenerationsChange(Number(val))}
                  size="sm"
                  options={generationOptions}
                />
              </div>
            )}

            {/* Orientation Toggle Button */}
            {mode !== 'fan' && mode !== 'all' && onOrientationChange && (
              <div data-testid="mobile-tree-orientation-toggle" className="flex flex-col gap-1">
                <label className="text-xs font-medium text-secondary">
                  {t('tree.toolbar.orientationAria')}
                </label>
                <SegmentedControl<TreeLayoutOrientation>
                  ariaLabel={t('tree.toolbar.orientationAria')}
                  value={orientation}
                  onChange={onOrientationChange}
                  size="sm"
                  options={[
                    {
                      value: 'vertical',
                      label: t('tree.toolbar.orientationVertical'),
                      icon: <ArrowUpDown className="h-3.5 w-3.5" />,
                    },
                    {
                      value: 'horizontal',
                      label: t('tree.toolbar.orientationHorizontal'),
                      icon: <ArrowLeftRight className="h-3.5 w-3.5" />,
                    },
                  ]}
                />
              </div>
            )}

            {/* Collateral Toggle Button (Hourglass mode) */}
            {mode === 'hourglass' && onCollateralChange && (
              <div data-testid="mobile-hourglass-collateral-toggle" className="flex flex-col gap-1">
                <label className="text-xs font-medium text-secondary">
                  {t('tree.view_options.collateral_toggle')}
                </label>
                <SegmentedControl<'wide' | 'compact'>
                  ariaLabel={t('tree.view_options.collateral_toggle')}
                  value={collateral ? 'wide' : 'compact'}
                  onChange={(val) => onCollateralChange(val === 'wide')}
                  size="sm"
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
                <span className="text-[11px] text-muted">
                  {t('tree.view_options.collateral_subtitle')}
                </span>
              </div>
            )}
          </div>
        </div>

        {/* Desktop View (>= 640px): Standard floating multi-pill bar */}
        <div className="hidden sm:flex flex-wrap items-center gap-1.5 sm:gap-2">
          <div
            onPointerDown={(e) => e.stopPropagation()}
            className="pointer-events-auto self-start h-11 flex items-center w-36 sm:w-44 rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
          >
            <Select<TreeProjectionMode>
              ariaLabel={t('tree.toolbar.modeSwitcherAria')}
              value={mode}
              onChange={onModeChange}
              size="sm"
              options={modeOptions}
            />
          </div>

          {mode === 'hourglass' && onCollateralChange && (
            <div
              onPointerDown={(e) => e.stopPropagation()}
              data-testid="hourglass-collateral-toggle"
              title={t('tree.view_options.collateral_subtitle')}
              className="pointer-events-auto self-start h-11 flex items-center max-w-full overflow-x-auto rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
            >
              <SegmentedControl<'wide' | 'compact'>
                ariaLabel={t('tree.view_options.collateral_toggle')}
                value={collateral ? 'wide' : 'compact'}
                onChange={(val) => onCollateralChange(val === 'wide')}
                size="sm"
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

          {mode !== 'fan' && mode !== 'all' && onOrientationChange && (
            <div
              onPointerDown={(e) => e.stopPropagation()}
              data-testid="tree-orientation-toggle"
              title={t('tree.toolbar.orientationAria')}
              className="pointer-events-auto self-start h-11 flex items-center max-w-full overflow-x-auto rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
            >
              <SegmentedControl<TreeLayoutOrientation>
                ariaLabel={t('tree.toolbar.orientationAria')}
                value={orientation}
                onChange={onOrientationChange}
                size="sm"
                options={[
                  {
                    value: 'vertical',
                    label: t('tree.toolbar.orientationVertical'),
                    icon: <ArrowUpDown className="h-3.5 w-3.5" />,
                  },
                  {
                    value: 'horizontal',
                    label: t('tree.toolbar.orientationHorizontal'),
                    icon: <ArrowLeftRight className="h-3.5 w-3.5" />,
                  },
                ]}
              />
            </div>
          )}
        </div>

        {/* Depth & Framing Controls */}
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="hidden sm:flex pointer-events-auto self-start sm:self-auto h-11 items-center gap-1.5 sm:gap-2 rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
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
            <div className="w-32 sm:w-40">
              <Select
                ariaLabel={t('tree.toolbar.generationsAria')}
                value={String(generations)}
                onChange={(val) => onGenerationsChange(Number(val))}
                size="sm"
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

      {/* Bottom-Left Floating Zoom Pill — matching exact 44px (h-11) height of other toolbars */}
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="absolute bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] md:bottom-6 left-3 sm:left-6 z-20 flex h-11 items-center gap-1 rounded-xl border border-border bg-surface/90 p-1 shadow-sm backdrop-blur-md"
      >
        <IconButton
          size="sm"
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
          className="min-w-12 h-full font-mono text-xs font-semibold px-1"
        >
          {Math.round(zoom * 100)}%
        </Button>

        <IconButton
          size="sm"
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
