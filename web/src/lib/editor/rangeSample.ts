import type { PixelGrid } from '#lib/editor/targetPick.ts';
import {
  oklabDistance,
  oklabToDisplay,
  displayToOklab,
  type Oklab,
  type Rgb
} from '#lib/utils/oklab.ts';

export type RangeSample = {
  rgb: Rgb;
  lab: Oklab;
  lo: number;
  hi: number;
  spread: number;
};

export type LumaWindow = { min: number; max: number };

export type ColorWindow = { sample_rgb: [number, number, number]; tolerance: number };

const LOW_PERCENTILE = 0.05;
const HIGH_PERCENTILE = 0.95;
const SPREAD_PERCENTILE = 0.9;
const STEP = 0.01;

export function displayLuma([r, g, b]: Rgb): number {
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return 0;
  const at = Math.min(sorted.length - 1, Math.max(0, Math.round(p * (sorted.length - 1))));
  return sorted[at] ?? 0;
}

function pixels(grid: PixelGrid): Rgb[] {
  const out: Rgb[] = [];
  for (let i = 0; i + 2 < grid.data.length; i += 4) {
    out.push([
      (grid.data[i] ?? 0) / 255,
      (grid.data[i + 1] ?? 0) / 255,
      (grid.data[i + 2] ?? 0) / 255
    ]);
  }
  return out;
}

function meanLab(labs: readonly Oklab[]): Oklab {
  const n = Math.max(labs.length, 1);
  return labs.reduce(
    (sum, lab) => ({ l: sum.l + lab.l / n, a: sum.a + lab.a / n, b: sum.b + lab.b / n }),
    {
      l: 0,
      a: 0,
      b: 0
    }
  );
}

export function pointSample(grid: PixelGrid): RangeSample {
  const lab = meanLab(pixels(grid).map(displayToOklab));
  const rgb = oklabToDisplay(lab);
  const luma = displayLuma(rgb);
  return { rgb, lab, lo: luma, hi: luma, spread: 0 };
}

export function areaSample(grid: PixelGrid): RangeSample {
  const all = pixels(grid);
  const lumas = all.map(displayLuma).sort((a, b) => a - b);
  const labs = all.map(displayToOklab);
  const lab = meanLab(labs);
  const spreads = labs.map((each) => oklabDistance(each, lab)).sort((a, b) => a - b);
  return {
    rgb: oklabToDisplay(lab),
    lab,
    lo: percentile(lumas, LOW_PERCENTILE),
    hi: percentile(lumas, HIGH_PERCENTILE),
    spread: percentile(spreads, SPREAD_PERCENTILE)
  };
}

function down(value: number): number {
  return Math.max(0, Math.floor(value / STEP + 1e-9) * STEP);
}

function up(value: number): number {
  return Math.min(1, Math.ceil(value / STEP - 1e-9) * STEP);
}

export function lumaWindow(samples: readonly RangeSample[]): LumaWindow | null {
  if (samples.length === 0) return null;
  const min = Math.min(...samples.map((s) => s.lo));
  const max = Math.max(...samples.map((s) => s.hi));
  return { min: Number(down(min).toFixed(2)), max: Number(up(max).toFixed(2)) };
}

export function colorWindow(
  samples: readonly RangeSample[],
  baseTolerance: number
): ColorWindow | null {
  if (samples.length === 0) return null;
  const centre = meanLab(samples.map((s) => s.lab));
  const reach = Math.max(...samples.map((s) => oklabDistance(s.lab, centre) + s.spread));
  return {
    sample_rgb: oklabToDisplay(centre),
    tolerance: Number(Math.max(baseTolerance, up(reach)).toFixed(2))
  };
}
