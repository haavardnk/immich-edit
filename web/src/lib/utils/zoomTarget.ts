import type { FaceBox } from '$lib/api/faces';
import { sceneUvToDisplayUv, type ViewTransform } from '$lib/utils/canvasCoords';
import type { Pan } from '$lib/utils/imageViewport';

export interface ZoomTarget {
  u: number;
  v: number;
}

const SAMPLE_EDGE = 192;
const GRID_CELLS = 8;
const MIN_ENERGY = 1.5;

export function faceTargets(faces: FaceBox[], view: ViewTransform): ZoomTarget[] {
  return faces
    .slice()
    .sort((a, b) => b.w * b.h - a.w * a.h)
    .map((face) => sceneUvToDisplayUv(view, face.x + face.w / 2, face.y + face.h / 2))
    .filter(([u, v]) => u >= 0 && u <= 1 && v >= 0 && v <= 1)
    .map(([u, v]) => ({ u, v }));
}

export function nextTargetIndex(index: number | null, count: number): number | null {
  if (count <= 0) return null;
  if (index === null) return 0;
  return index + 1 >= count ? null : index + 1;
}

export function panForTarget(
  target: ZoomTarget,
  frame: { width: number; height: number },
  viewW: number,
  viewH: number,
  zoomRatio: number
): Pan {
  const width = frame.width * zoomRatio;
  const height = frame.height * zoomRatio;
  const limitX = Math.max(0, (width - viewW) / 2);
  const limitY = Math.max(0, (height - viewH) / 2);
  return {
    panX: Math.min(limitX, Math.max(-limitX, (0.5 - target.u) * width)),
    panY: Math.min(limitY, Math.max(-limitY, (0.5 - target.v) * height))
  };
}

export function sharpestCell(
  gray: Float32Array,
  w: number,
  h: number,
  cols = GRID_CELLS,
  rows = GRID_CELLS
): ZoomTarget | null {
  if (w < 3 || h < 3 || gray.length < w * h) return null;
  const at = (i: number): number => gray[i] ?? 0;
  const sums = new Float32Array(cols * rows);
  const counts = new Uint32Array(cols * rows);
  for (let y = 1; y < h - 1; y++) {
    const row = Math.min(rows - 1, Math.floor((y / h) * rows));
    for (let x = 1; x < w - 1; x++) {
      const i = y * w + x;
      const cell = row * cols + Math.min(cols - 1, Math.floor((x / w) * cols));
      sums[cell] =
        (sums[cell] ?? 0) + Math.abs(4 * at(i) - at(i - 1) - at(i + 1) - at(i - w) - at(i + w));
      counts[cell] = (counts[cell] ?? 0) + 1;
    }
  }
  let best = -1;
  let bestEnergy = 0;
  for (let cell = 0; cell < sums.length; cell++) {
    const count = counts[cell] ?? 0;
    if (count === 0) continue;
    const energy = (sums[cell] ?? 0) / count;
    if (energy > bestEnergy) {
      best = cell;
      bestEnergy = energy;
    }
  }
  if (best < 0 || bestEnergy < MIN_ENERGY) return null;
  return {
    u: ((best % cols) + 0.5) / cols,
    v: (Math.floor(best / cols) + 0.5) / rows
  };
}

export function sharpestPoint(image: HTMLImageElement): ZoomTarget | null {
  const nw = image.naturalWidth;
  const nh = image.naturalHeight;
  if (nw <= 0 || nh <= 0) return null;
  const scale = Math.min(1, SAMPLE_EDGE / Math.max(nw, nh));
  const w = Math.max(3, Math.round(nw * scale));
  const h = Math.max(3, Math.round(nh * scale));
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) return null;
  ctx.drawImage(image, 0, 0, w, h);
  let pixels: Uint8ClampedArray;
  try {
    pixels = ctx.getImageData(0, 0, w, h).data;
  } catch {
    return null;
  }
  const gray = new Float32Array(w * h);
  for (let i = 0; i < gray.length; i++) {
    const p = i * 4;
    gray[i] =
      0.299 * (pixels[p] ?? 0) + 0.587 * (pixels[p + 1] ?? 0) + 0.114 * (pixels[p + 2] ?? 0);
  }
  return sharpestCell(gray, w, h);
}
