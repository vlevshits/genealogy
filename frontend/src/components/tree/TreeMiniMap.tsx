import React, { useState } from 'react';
import { Map as MapIcon, Minimize2 } from 'lucide-react';
import { Button, IconButton } from '../ui';
import { LayoutBounds, PositionedNode } from '../../lib/tree-layouts';
import { useI18n } from '../../i18n';

export interface TreeMiniMapProps {
  nodes: PositionedNode[];
  bounds: LayoutBounds;
  pan: { x: number; y: number };
  zoom: number;
  viewportWidth: number;
  viewportHeight: number;
  onPanToWorldPoint?: (worldX: number, worldY: number) => void;
  defaultCollapsed?: boolean;
}

export const TreeMiniMap: React.FC<TreeMiniMapProps> = ({
  nodes,
  bounds,
  pan,
  zoom,
  viewportWidth,
  viewportHeight,
  onPanToWorldPoint,
  defaultCollapsed = false,
}) => {
  const { t } = useI18n();
  const [collapsed, setCollapsed] = useState<boolean>(defaultCollapsed);

  const pad = 120;
  const worldMinX = bounds.minX - pad;
  const worldMinY = bounds.minY - pad;
  const worldW = Math.max(400, bounds.width + pad * 2);
  const worldH = Math.max(300, bounds.height + pad * 2);

  // Camera viewport rectangle in world coordinates
  const camWorldX = -pan.x / zoom;
  const camWorldY = -pan.y / zoom;
  const camWorldW = viewportWidth / zoom;
  const camWorldH = viewportHeight / zoom;

  const handleSvgPointerDown = (e: React.PointerEvent<SVGSVGElement>) => {
    e.stopPropagation();
    if (!onPanToWorldPoint) return;
    const rect = e.currentTarget.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;
    const ratioX = (e.clientX - rect.left) / rect.width;
    const ratioY = (e.clientY - rect.top) / rect.height;
    const targetWorldX = worldMinX + ratioX * worldW;
    const targetWorldY = worldMinY + ratioY * worldH;
    onPanToWorldPoint(targetWorldX, targetWorldY);
  };

  if (collapsed) {
    return (
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="absolute bottom-6 right-4 sm:right-6 z-20"
      >
        <Button
          size="md"
          variant="secondary"
          leftIcon={<MapIcon className="h-4 w-4" />}
          onClick={() => setCollapsed(false)}
          className="rounded-xl border border-border bg-surface/90 shadow-sm backdrop-blur-md"
        >
          {t('tree.miniMap.openBtn')}
        </Button>
      </div>
    );
  }

  return (
    <div
      aria-label={t('tree.miniMap.title')}
      onPointerDown={(e) => e.stopPropagation()}
      className="absolute bottom-6 right-4 sm:right-6 z-20 w-48 h-32 rounded-xl border border-border bg-surface/90 p-2 shadow-sm backdrop-blur-md flex flex-col justify-between"
    >
      <div className="flex items-center justify-between gap-1 text-[11px] font-medium text-secondary">
        <span className="truncate">{t('tree.miniMap.title')}</span>
        <IconButton
          size="sm"
          variant="ghost"
          label={t('tree.miniMap.collapseAria')}
          onClick={() => setCollapsed(true)}
          className="h-6 w-6 min-h-6 min-w-6"
        >
          <Minimize2 className="h-3 w-3" />
        </IconButton>
      </div>

      <svg
        viewBox={`${worldMinX} ${worldMinY} ${worldW} ${worldH}`}
        onPointerDown={handleSvgPointerDown}
        className="w-full h-20 rounded-lg bg-canvas/80 border border-border/60 cursor-crosshair"
      >
        {nodes.map((pn) => (
          <rect
            key={pn.id}
            x={pn.x}
            y={pn.y}
            width={pn.width}
            height={pn.height}
            rx={16}
            className={
              pn.isFocal
                ? 'fill-brand stroke-brand'
                : 'fill-secondary/50 stroke-border'
            }
          />
        ))}

        <rect
          x={camWorldX}
          y={camWorldY}
          width={camWorldW}
          height={camWorldH}
          rx={12}
          className="fill-brand/10 stroke-brand stroke-[14]"
        />
      </svg>
    </div>
  );
};
