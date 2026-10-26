export type BokehShape = 'circle' | 'hexagon' | 'octagon';

export const BOKEH_SHAPES: readonly BokehShape[] = ['circle', 'hexagon', 'octagon'];

export interface LensBlurEdits {
  amount: number;
  focus_min: number;
  focus_max: number;
  shape: BokehShape;
  boost: number;
  depth_raster_id: string;
}

export const LENS_BLUR_AMOUNT = 50;

export const NEUTRAL_LENS_BLUR: LensBlurEdits = {
  amount: 0,
  focus_min: 0,
  focus_max: 0.3,
  shape: 'circle',
  boost: 0,
  depth_raster_id: ''
};

export function lensBlurActive(e: LensBlurEdits): boolean {
  return e.amount > 0 && e.depth_raster_id !== '';
}

export function resetLensBlur(e: LensBlurEdits): LensBlurEdits {
  return { ...NEUTRAL_LENS_BLUR, depth_raster_id: e.depth_raster_id };
}
