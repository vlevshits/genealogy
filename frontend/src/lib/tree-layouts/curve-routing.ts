/**
 * Pure SVG path generators for genealogical tree connectors:
 * - Horizontal cubic Bezier curves for Pedigree ancestor branches
 * - Vertical orthogonal bus / smooth elbow curves for Hourglass & Descendants
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
  return `M ${sourceX} ${sourceY} L ${sourceX} ${midY} L ${targetX} ${midY} L ${targetX} ${targetY}`;
}

export function buildSpouseConnectorPath(
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number
): string {
  return `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`;
}
