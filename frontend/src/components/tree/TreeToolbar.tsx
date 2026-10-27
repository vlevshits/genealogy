import React from 'react';
import {
  GitBranch,
  GitFork,
  Home,
  Maximize2,
  Minus,
  Plus,
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
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onZoomReset: () => void;
  onCenterFocal: () => void;
  onFitToScreen: () => void;
}

export const TreeToolbar: React.FC<TreeToolbarProps> = ({
  mode,
  onModeChange,
  generations,
  onGenerationsChange,
  zoom,
  onZoomIn,
  onZoomOut,
  onZoomReset,
  onCenterFocal,
  onFitToScreen,
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
      <div className="pointer-events-none absolute top-3 inset-x-3 sm:top-4 sm:inset-x-6 z-20 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        {/* Projection Mode Switcher */}
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
            ]}
          />
        </div>

        {/* Depth & Framing Controls */}
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="pointer-events-auto self-start sm:self-auto flex items-center gap-1.5 sm:gap-2 rounded-xl border border-border bg-surface/90 p-1.5 shadow-sm backdrop-blur-md"
        >
          <div className="w-36 sm:w-44">
            <Select
              ariaLabel={t('tree.toolbar.generationsAria')}
              value={String(generations)}
              onChange={(val) => onGenerationsChange(Number(val))}
              options={generationOptions}
            />
          </div>

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
