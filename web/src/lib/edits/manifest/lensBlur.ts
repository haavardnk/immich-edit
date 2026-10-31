import {
  BOKEH_SHAPES,
  NEUTRAL_LENS_BLUR,
  type BokehShape,
  type Edits,
  type LensBlurEdits
} from '#lib/types/edits.ts';

const LENS_BLUR_KEYS = Object.keys(NEUTRAL_LENS_BLUR) as (keyof LensBlurEdits)[];

function isNeutral(blur: LensBlurEdits): boolean {
  return LENS_BLUR_KEYS.every((key) => blur[key] === NEUTRAL_LENS_BLUR[key]);
}

export function encodeLensBlur(edits: Edits, ops: Record<string, unknown>): void {
  if (isNeutral(edits.lens_blur)) return;
  ops.lens_blur = { ...edits.lens_blur };
}

function number(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function shape(value: unknown): BokehShape {
  return BOKEH_SHAPES.find((s) => s === value) ?? NEUTRAL_LENS_BLUR.shape;
}

export function decodeLensBlur(ops: Record<string, unknown>, edits: Edits): void {
  const raw = ops.lens_blur;
  if (!raw || typeof raw !== 'object') return;
  const doc = raw as Record<string, unknown>;
  edits.lens_blur = {
    amount: number(doc.amount, NEUTRAL_LENS_BLUR.amount),
    focus_min: number(doc.focus_min, NEUTRAL_LENS_BLUR.focus_min),
    focus_max: number(doc.focus_max, NEUTRAL_LENS_BLUR.focus_max),
    shape: shape(doc.shape),
    boost: number(doc.boost, NEUTRAL_LENS_BLUR.boost),
    depth_raster_id:
      typeof doc.depth_raster_id === 'string'
        ? doc.depth_raster_id
        : NEUTRAL_LENS_BLUR.depth_raster_id
  };
}
