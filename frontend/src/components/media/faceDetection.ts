import { FaceTagDto } from '../../api/client';

export interface NormalizedBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface DetectedFace {
  boundingBox: DOMRectReadOnly;
}

interface WindowWithFaceDetector extends Window {
  FaceDetector?: new (options?: {
    fastMode?: boolean;
    maxDetectedFaces?: number;
  }) => {
    detect(image: ImageBitmapSource): Promise<DetectedFace[]>;
  };
}

/**
 * Computes Intersection over Union (IoU) of two normalized 0.0..1.0 bounding boxes.
 */
export function computeIoU(a: NormalizedBox, b: NormalizedBox): number {
  const x1 = Math.max(a.x, b.x);
  const y1 = Math.max(a.y, b.y);
  const x2 = Math.min(a.x + a.width, b.x + b.width);
  const y2 = Math.min(a.y + a.height, b.y + b.height);

  const interWidth = Math.max(0, x2 - x1);
  const interHeight = Math.max(0, y2 - y1);
  const interArea = interWidth * interHeight;

  const areaA = a.width * a.height;
  const areaB = b.width * b.height;
  const unionArea = areaA + areaB - interArea;

  if (unionArea <= 0) return 0;
  return interArea / unionArea;
}

/**
 * Filters out candidate boxes that overlap with existing confirmed face tags
 * (IoU >= threshold or centroid containment) and deduplicates candidates.
 */
export function filterOverlappingCandidates(
  candidates: NormalizedBox[],
  existingTags: FaceTagDto[],
  iouThreshold = 0.3
): NormalizedBox[] {
  const filtered: NormalizedBox[] = [];

  for (const cand of candidates) {
    const overlapsExisting = existingTags.some((tag) => {
      const tagBox: NormalizedBox = {
        x: tag.x,
        y: tag.y,
        width: tag.width,
        height: tag.height,
      };
      if (computeIoU(cand, tagBox) >= iouThreshold) return true;

      // Also check centroid containment to prevent small boxes inside large boxes
      const cx = cand.x + cand.width / 2;
      const cy = cand.y + cand.height / 2;
      return (
        cx >= tagBox.x &&
        cx <= tagBox.x + tagBox.width &&
        cy >= tagBox.y &&
        cy <= tagBox.y + tagBox.height
      );
    });

    if (overlapsExisting) continue;

    // Deduplicate against candidates already selected to prevent visual clutter
    const duplicateOfAccepted = filtered.some(
      (accepted) => computeIoU(cand, accepted) >= 0.45
    );
    if (!duplicateOfAccepted) {
      filtered.push(cand);
    }
  }

  return filtered;
}

/**
 * Lightweight in-browser fallback face clustering heuristic using offscreen canvas.
 * Analyzes chromatic skin-tone distribution and contrast across a 16x12 grid.
 */
export function detectFacesHeuristic(
  imgElement: HTMLImageElement
): NormalizedBox[] {
  if (typeof document === 'undefined') return [];

  try {
    const canvas = document.createElement('canvas');
    const targetW = 320;
    const targetH = 240;
    canvas.width = targetW;
    canvas.height = targetH;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return [];

    ctx.drawImage(imgElement, 0, 0, targetW, targetH);
    const imgData = ctx.getImageData(0, 0, targetW, targetH);
    const data = imgData.data;

    const cols = 16;
    const rows = 12;
    const cellW = targetW / cols;
    const cellH = targetH / rows;

    const skinGrid: number[][] = Array.from({ length: rows }, () =>
      Array(cols).fill(0)
    );

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        let skinCount = 0;
        let total = 0;
        const startX = Math.floor(c * cellW);
        const startY = Math.floor(r * cellH);
        const endX = Math.floor((c + 1) * cellW);
        const endY = Math.floor((r + 1) * cellH);

        for (let y = startY; y < endY; y += 2) {
          for (let x = startX; x < endX; x += 2) {
            const idx = (y * targetW + x) * 4;
            const red = data[idx];
            const green = data[idx + 1];
            const blue = data[idx + 2];
            total++;

            // Peer / Kovac skin chromaticity model:
            const maxVal = Math.max(red, green, blue);
            const minVal = Math.min(red, green, blue);
            if (
              red > 95 &&
              green > 40 &&
              blue > 20 &&
              maxVal - minVal > 15 &&
              Math.abs(red - green) > 15 &&
              red > green &&
              red > blue
            ) {
              skinCount++;
            }
          }
        }
        skinGrid[r][c] = total > 0 ? skinCount / total : 0;
      }
    }

    // Connect contiguous skin clusters
    const visited: boolean[][] = Array.from({ length: rows }, () =>
      Array(cols).fill(false)
    );
    const clusters: Array<{
      minC: number;
      maxC: number;
      minR: number;
      maxR: number;
      cells: number;
    }> = [];

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        if (!visited[r][c] && skinGrid[r][c] >= 0.35) {
          let minC = c;
          let maxC = c;
          let minR = r;
          let maxR = r;
          let cells = 0;
          const queue: Array<[number, number]> = [[r, c]];
          visited[r][c] = true;

          while (queue.length > 0) {
            const [currR, currC] = queue.shift()!;
            cells++;
            minC = Math.min(minC, currC);
            maxC = Math.max(maxC, currC);
            minR = Math.min(minR, currR);
            maxR = Math.max(maxR, currR);

            const neighbors: Array<[number, number]> = [
              [currR - 1, currC],
              [currR + 1, currC],
              [currR, currC - 1],
              [currR, currC + 1],
            ];

            for (const [nr, nc] of neighbors) {
              if (
                nr >= 0 &&
                nr < rows &&
                nc >= 0 &&
                nc < cols &&
                !visited[nr][nc] &&
                skinGrid[nr][nc] >= 0.35
              ) {
                visited[nr][nc] = true;
                queue.push([nr, nc]);
              }
            }
          }

          // Typical face bounds span 2 to 14 cells
          if (cells >= 2 && cells <= 14) {
            clusters.push({ minC, maxC, minR, maxR, cells });
          }
        }
      }
    }

    return clusters.map((cl) => {
      const x = Math.max(0, (cl.minC * cellW) / targetW);
      const y = Math.max(0, (cl.minR * cellH) / targetH);
      const w = Math.min(1 - x, ((cl.maxC - cl.minC + 1) * cellW) / targetW);
      const h = Math.min(1 - y, ((cl.maxR - cl.minR + 1) * cellH) / targetH);
      return {
        x: Math.round(x * 1000) / 1000,
        y: Math.round(y * 1000) / 1000,
        width: Math.max(0.08, Math.round(w * 1000) / 1000),
        height: Math.max(0.08, Math.round(h * 1000) / 1000),
      };
    });
  } catch {
    return [];
  }
}

/**
 * Detects faces in an image element using the browser's native Shape Detection API
 * (FaceDetector) when available, falling back to client-side heuristics.
 */
export async function detectFacesInImage(
  imgElement: HTMLImageElement
): Promise<NormalizedBox[]> {
  if (typeof window === 'undefined' || !imgElement) {
    return [];
  }

  const win = window as WindowWithFaceDetector;

  // Modern browser Shape Detection API
  if (typeof win.FaceDetector === 'function') {
    try {
      const detector = new win.FaceDetector({
        fastMode: true,
        maxDetectedFaces: 20,
      });
      const faces = await detector.detect(imgElement);

      const naturalW = imgElement.naturalWidth || imgElement.width || 1;
      const naturalH = imgElement.naturalHeight || imgElement.height || 1;

      if (faces && faces.length > 0) {
        return faces.map((face) => ({
          x: Math.max(0, Math.min(1, face.boundingBox.x / naturalW)),
          y: Math.max(0, Math.min(1, face.boundingBox.y / naturalH)),
          width: Math.max(
            0.02,
            Math.min(1, face.boundingBox.width / naturalW)
          ),
          height: Math.max(
            0.02,
            Math.min(1, face.boundingBox.height / naturalH)
          ),
        }));
      }
    } catch {
      // Security error, cross-origin taint, or failure: proceed to heuristic fallback
    }
  }

  // Fallback heuristic
  return detectFacesHeuristic(imgElement);
}
