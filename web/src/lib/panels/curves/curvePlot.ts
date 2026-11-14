import type { CurveChannel, CurvePoint } from '#lib/types/edits.ts';
import type { Histogram } from '#lib/types/preview.ts';
import { hermite } from './curves';

export const PLOT_ASPECT = 1.5;
export const PLOT_PAD = 10;
export const MOUSE_HIT = 12;
export const TOUCH_HIT = 22;

const PATH_STEPS = 64;
const HIST_HEIGHT = 0.8;
const LEVELS = 255;

export type Plot = { width: number; height: number; pad: number };
export type SvgPoint = { x: number; y: number };
export type GridLine = { x1: number; y1: number; x2: number; y2: number };
export type HistLayer = { d: string; fill: string };
type HistSource = { key: keyof Histogram; fill: string };

const CHANNEL_HIST: Record<Exclude<CurveChannel, 'composite'>, HistSource> = {
  r: { key: 'r', fill: 'color-mix(in srgb, var(--color-curve-red) 35%, transparent)' },
  g: { key: 'g', fill: 'color-mix(in srgb, var(--color-curve-green) 35%, transparent)' },
  b: { key: 'b', fill: 'color-mix(in srgb, var(--color-curve-blue) 35%, transparent)' },
  luma: { key: 'l', fill: 'color-mix(in srgb, var(--color-channel-luma) 18%, transparent)' }
};

const COMPOSITE_HIST: HistSource[] = [
  { key: 'r', fill: 'color-mix(in srgb, var(--color-curve-red) 22%, transparent)' },
  { key: 'g', fill: 'color-mix(in srgb, var(--color-curve-green) 22%, transparent)' },
  { key: 'b', fill: 'color-mix(in srgb, var(--color-curve-blue) 22%, transparent)' }
];

export function plotOf(width: number): Plot {
  return { width, height: Math.round(width / PLOT_ASPECT), pad: PLOT_PAD };
}

function innerWidth(plot: Plot): number {
  return Math.max(1, plot.width - 2 * plot.pad);
}

function innerHeight(plot: Plot): number {
  return Math.max(1, plot.height - 2 * plot.pad);
}

export function toSvg(plot: Plot, p: CurvePoint): SvgPoint {
  return {
    x: plot.pad + p.x * innerWidth(plot),
    y: plot.pad + (1 - p.y) * innerHeight(plot)
  };
}

export function fromSvg(plot: Plot, sx: number, sy: number): CurvePoint {
  const x = Math.max(0, Math.min(1, (sx - plot.pad) / innerWidth(plot)));
  const y = Math.max(0, Math.min(1, 1 - (sy - plot.pad) / innerHeight(plot)));
  return { x, y };
}

export function gridLines(plot: Plot): GridLine[] {
  const right = plot.pad + innerWidth(plot);
  const bottom = plot.pad + innerHeight(plot);
  return [1, 2, 3].flatMap((i) => {
    const x = plot.pad + (i / 4) * innerWidth(plot);
    const y = plot.pad + (i / 4) * innerHeight(plot);
    return [
      { x1: x, y1: plot.pad, x2: x, y2: bottom },
      { x1: plot.pad, y1: y, x2: right, y2: y }
    ];
  });
}

export function curvePath(plot: Plot, pts: CurvePoint[]): string {
  if (pts.length < 2) return '';
  return Array.from({ length: PATH_STEPS + 1 }, (_, i) => {
    const t = i / PATH_STEPS;
    const sv = toSvg(plot, { x: t, y: hermite(pts, t) });
    return `${i === 0 ? 'M' : 'L'}${sv.x.toFixed(1)},${sv.y.toFixed(1)}`;
  }).join(' ');
}

function histAreaPath(plot: Plot, data: number[] | undefined): string {
  if (!data || data.length < 2) return '';
  const max = Math.max(...data, 1);
  const left = plot.pad;
  const right = plot.pad + innerWidth(plot);
  const base = plot.pad + innerHeight(plot);
  const steps = data.map((count, i) => {
    const x = left + (i / (data.length - 1)) * innerWidth(plot);
    const y = base - (count / max) * innerHeight(plot) * HIST_HEIGHT;
    return ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
  });
  return `M ${left} ${base}${steps.join('')} L ${right} ${base} Z`;
}

export function histLayers(plot: Plot, hist: Histogram | null, channel: CurveChannel): HistLayer[] {
  if (!hist) return [];
  const sources = channel === 'composite' ? COMPOSITE_HIST : [CHANNEL_HIST[channel]];
  return sources.map(({ key, fill }) => ({ d: histAreaPath(plot, hist[key]), fill }));
}

export function hitTest(
  plot: Plot,
  pts: CurvePoint[],
  at: SvgPoint,
  radius: number,
  interior = false
): number {
  let best = -1;
  let bestDistance = radius;
  pts.forEach((p, i) => {
    if (interior && (i === 0 || i === pts.length - 1)) return;
    const sp = toSvg(plot, p);
    const d = Math.hypot(at.x - sp.x, at.y - sp.y);
    if (d < bestDistance) {
      best = i;
      bestDistance = d;
    }
  });
  return best;
}

export function levelOf(value: number): number {
  return Math.round(value * LEVELS);
}

export function fromLevel(level: number): number {
  return Math.max(0, Math.min(LEVELS, level)) / LEVELS;
}
