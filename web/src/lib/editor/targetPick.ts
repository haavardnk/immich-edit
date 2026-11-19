import type { GradeRegion } from '#lib/editor/params/colorGrading.ts';
import { BW_CHANNELS, type BwChannel, type CurveChannel } from '#lib/types/edits.ts';

export type Rgb = readonly [number, number, number];

export interface PixelGrid {
  width: number;
  height: number;
  data: Uint8ClampedArray;
}

const SAMPLE_RADIUS = 2;

export const HSL_BAND_CENTRES: readonly number[] = [0, 30, 60, 120, 180, 240, 270, 300];

const HSL_SAT_GATE = 0.05;
const BW_CHROMA_GATE = 0.05;
const GRADE_PIVOT_BASE = 0.5;
const GRADE_PIVOT_RANGE = 0.3;
const GRADE_FEATHER_BASE = 0.15;
const GRADE_FEATHER_RANGE = 0.25;

function toLinear(channel: number): number {
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function linear([r, g, b]: Rgb): Rgb {
  return [toLinear(r), toLinear(g), toLinear(b)];
}

function hueOf(r: number, g: number, b: number, max: number, d: number): number {
  const sector =
    max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return sector * 60;
}

function hueDistance(a: number, b: number): number {
  const d = Math.abs(a - b) % 360;
  return d > 180 ? 360 - d : d;
}

function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = Math.min(1, Math.max(0, (x - edge0) / Math.max(edge1 - edge0, 1e-6)));
  return t * t * (3 - 2 * t);
}

export function hslBandAt(rgb: Rgb): number | null {
  const [r, g, b] = linear(rgb);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  if (d < 1e-6) return null;
  const s = max + min > 1 ? d / (2 - max - min) : d / (max + min);
  if (s < HSL_SAT_GATE) return null;
  const hue = hueOf(r, g, b, max, d);
  return HSL_BAND_CENTRES.reduce(
    (best, centre, band) =>
      hueDistance(hue, centre) < hueDistance(hue, HSL_BAND_CENTRES[best] ?? 0) ? band : best,
    0
  );
}

export function bwChannelAt(rgb: Rgb): BwChannel | null {
  const [r, g, b] = linear(rgb);
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  if (max < 1e-6 || d / max < BW_CHROMA_GATE) return null;
  const sector = Math.round(hueOf(r, g, b, max, d) / 60) % BW_CHANNELS.length;
  return BW_CHANNELS[sector] ?? null;
}

function oklabLightness([r, g, b]: Rgb): number {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
}

export function gradeRegionAt(
  rgb: Rgb,
  balance: number,
  blend: number
): Exclude<GradeRegion, 'global'> {
  const y = Math.min(1, Math.max(0, oklabLightness(linear(rgb))));
  const pivot = GRADE_PIVOT_BASE + GRADE_PIVOT_RANGE * (balance / 100);
  const feather = GRADE_FEATHER_BASE + GRADE_FEATHER_RANGE * (blend / 100);
  const sHi = Math.min(0.999, Math.max(0.001, pivot + feather * 0.5));
  const sLo = Math.min(sHi - 0.001, Math.max(0, pivot - feather * 1.5));
  const hLo = Math.min(0.999, Math.max(0.001, pivot - feather * 0.5));
  const hHi = Math.min(1, Math.max(hLo + 0.001, pivot + feather * 1.5));
  const shadow = 1 - smoothstep(sLo, sHi, y);
  const highlight = smoothstep(hLo, hHi, y);
  const mid = Math.max(0, 1 - shadow - highlight);
  if (shadow >= mid && shadow >= highlight) return 'shadows';
  return highlight > mid ? 'highlights' : 'midtones';
}

export function toneAt([r, g, b]: Rgb, channel: CurveChannel): number {
  if (channel === 'r') return r;
  if (channel === 'g') return g;
  if (channel === 'b') return b;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function averageAt(grid: PixelGrid, u: number, v: number, radius = SAMPLE_RADIUS): Rgb {
  const cx = Math.min(grid.width - 1, Math.max(0, Math.floor(u * grid.width)));
  const cy = Math.min(grid.height - 1, Math.max(0, Math.floor(v * grid.height)));
  const x0 = Math.max(0, cx - radius);
  const y0 = Math.max(0, cy - radius);
  const cols = Math.min(grid.width - 1, cx + radius) - x0 + 1;
  const count = cols * (Math.min(grid.height - 1, cy + radius) - y0 + 1);
  let r = 0;
  let g = 0;
  let b = 0;
  for (let k = 0; k < count; k++) {
    const i = ((y0 + Math.floor(k / cols)) * grid.width + x0 + (k % cols)) * 4;
    r += grid.data[i] ?? 0;
    g += grid.data[i + 1] ?? 0;
    b += grid.data[i + 2] ?? 0;
  }
  const scale = 1 / (Math.max(count, 1) * 255);
  return [r * scale, g * scale, b * scale];
}
