import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';

export type Detent = number | { fraction: number } | 'content';

export type DetentSpace = { available: number; content: number };

export type SheetFrame = { height: number; offset: number };

export type SnapTarget = number | 'dismiss';

export type SnapInput = {
  heights: readonly number[];
  height: number;
  velocity: number;
  dismissable: boolean;
  flingVelocity?: number;
  dismissFraction?: number;
};

type Entry = { height: number; index: number };

export const CONTENT_CAP = 0.9;
export const DISMISS_FRACTION = 0.4;
export const RUBBER_EXTENT = 120;
export const RUBBER_FACTOR = 0.55;

const SNAP_EPSILON = 1;

function clampTo(value: number, max: number): number {
  return Math.min(Math.max(0, value), Math.max(0, max));
}

function resolveDetent(detent: Detent, { available, content }: DetentSpace): number {
  if (detent === 'content') return clampTo(content, available * CONTENT_CAP);
  if (typeof detent === 'number') return clampTo(detent, available);
  return clampTo(detent.fraction, 1) * Math.max(0, available);
}

export function resolveDetents(detents: readonly Detent[], space: DetentSpace): number[] {
  return detents.map((detent) => resolveDetent(detent, space));
}

export function rubberBand(
  overshoot: number,
  extent = RUBBER_EXTENT,
  factor = RUBBER_FACTOR
): number {
  if (overshoot <= 0 || extent <= 0) return 0;
  return extent * (1 - 1 / ((overshoot * factor) / extent + 1));
}

export function dragFrame(
  raw: number,
  heights: readonly number[],
  dismissable: boolean
): SheetFrame {
  if (heights.length === 0) return { height: Math.max(0, raw), offset: 0 };
  const lowest = Math.min(...heights);
  const top = Math.max(...heights);
  if (raw > top) return { height: top + rubberBand(raw - top), offset: 0 };
  if (raw >= lowest) return { height: raw, offset: 0 };
  const below = lowest - raw;
  return { height: lowest, offset: dismissable ? below : rubberBand(below) };
}

function ascending(heights: readonly number[]): Entry[] {
  return heights.map((height, index) => ({ height, index })).sort((a, b) => a.height - b.height);
}

function nearest(entries: readonly Entry[], height: number, fallback: Entry): Entry {
  return entries.reduce(
    (best, entry) =>
      Math.abs(entry.height - height) < Math.abs(best.height - height) ? entry : best,
    fallback
  );
}

export function snapSheet({
  heights,
  height,
  velocity,
  dismissable,
  flingVelocity = DEFAULT_THRESHOLDS.flingVelocity,
  dismissFraction = DISMISS_FRACTION
}: SnapInput): SnapTarget {
  const entries = ascending(heights);
  const lowest = entries[0];
  if (!lowest) return dismissable ? 'dismiss' : 0;
  if (velocity > flingVelocity) {
    const below = entries.findLast((entry) => entry.height < height - SNAP_EPSILON);
    if (below) return below.index;
    return dismissable ? 'dismiss' : lowest.index;
  }
  if (velocity < -flingVelocity) {
    const above = entries.find((entry) => entry.height > height + SNAP_EPSILON);
    return (above ?? entries.at(-1) ?? lowest).index;
  }
  if (dismissable && lowest.height - height > lowest.height * dismissFraction) return 'dismiss';
  return nearest(entries, height, lowest).index;
}

export function nextDetent(index: number, count: number): number {
  if (count <= 0) return 0;
  return (index + 1) % count;
}
