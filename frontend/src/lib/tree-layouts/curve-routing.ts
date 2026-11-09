/**
 * Pure SVG path generators for genealogical tree connectors:
 * - Horizontal cubic Bezier curves for classic horizontal Pedigree ancestor branches
 * - Vertical orthogonal bus with smooth filleted elbow curves for Hourglass & Descendants
 * - Horizontal orthogonal bus with smooth filleted elbow curves
 * - Spouse union bars
 */

export function buildHorizontalBezierPath(
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number
): string {
  const midX = (sourceX + targetX) / 2;
  return `M ${sourceX} ${sourceY} C ${midX} ${sourceY}, ${midX} ${targetY}, ${targetX} ${targetY}`;
}

export function buildVerticalBusPath(
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number,
  customMidY?: number
): string {
  const midY = customMidY ?? (sourceY + targetY) / 2;
  const dx = targetX - sourceX;
  if (Math.abs(dx) < 1) {
    return `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`;
  }

  const dirX = dx > 0 ? 1 : -1;
  const dy1 = midY - sourceY;
  const dirY1 = dy1 >= 0 ? 1 : -1;
  const dy2 = targetY - midY;
  const dirY2 = dy2 >= 0 ? 1 : -1;

  // Fillet radius for seamless 90-degree corner
  const maxR = Math.min(
    10,
    Math.abs(dx) / 2,
    Math.abs(dy1) / 2,
    Math.abs(dy2) / 2
  );

  if (maxR < 2) {
    return `M ${sourceX} ${sourceY} L ${sourceX} ${midY} L ${targetX} ${midY} L ${targetX} ${targetY}`;
  }

  const r = maxR;
  return [
    `M ${sourceX} ${sourceY}`,
    `L ${sourceX} ${midY - dirY1 * r}`,
    `Q ${sourceX} ${midY}, ${sourceX + dirX * r} ${midY}`,
    `L ${targetX - dirX * r} ${midY}`,
    `Q ${targetX} ${midY}, ${targetX} ${midY + dirY2 * r}`,
    `L ${targetX} ${targetY}`,
  ].join(' ');
}

export function buildHorizontalBusPath(
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number,
  customMidX?: number
): string {
  const midX = customMidX ?? (sourceX + targetX) / 2;
  const dy = targetY - sourceY;
  if (Math.abs(dy) < 1) {
    return `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`;
  }

  const dirY = dy > 0 ? 1 : -1;
  const dx1 = midX - sourceX;
  const dirX1 = dx1 >= 0 ? 1 : -1;
  const dx2 = targetX - midX;
  const dirX2 = dx2 >= 0 ? 1 : -1;

  const maxR = Math.min(
    10,
    Math.abs(dy) / 2,
    Math.abs(dx1) / 2,
    Math.abs(dx2) / 2
  );

  if (maxR < 2) {
    return `M ${sourceX} ${sourceY} L ${midX} ${sourceY} L ${midX} ${targetY} L ${targetX} ${targetY}`;
  }

  const r = maxR;
  return [
    `M ${sourceX} ${sourceY}`,
    `L ${midX - dirX1 * r} ${sourceY}`,
    `Q ${midX} ${sourceY}, ${midX} ${sourceY + dirY * r}`,
    `L ${midX} ${targetY - dirY * r}`,
    `Q ${midX} ${targetY}, ${midX + dirX2 * r} ${targetY}`,
    `L ${targetX} ${targetY}`,
  ].join(' ');
}

export function buildSpouseConnectorPath(
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number
): string {
  return `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`;
}
