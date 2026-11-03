import {
  DEFAULT_COLOR_RANGE,
  DEFAULT_DEPTH_RANGE,
  DEFAULT_GENERATED_FEATHER,
  DEFAULT_GENERATED_GROW,
  DEFAULT_GRADIENT_FEATHER,
  DEFAULT_LUMA_RANGE,
  DEFAULT_POLYGON_FEATHER
} from '#lib/types/masks.ts';
import { thousandths, toolRange, whole } from './range';

export const MASK_FEATHER = toolRange(0, 1, 0.01, DEFAULT_GRADIENT_FEATHER);

export const POLYGON_FEATHER = toolRange(0, 0.5, 0.005, DEFAULT_POLYGON_FEATHER, {
  format: thousandths
});

export const LUMA_MIN = toolRange(0, 1, 0.01, DEFAULT_LUMA_RANGE.min);

export const LUMA_MAX = toolRange(0, 1, 0.01, DEFAULT_LUMA_RANGE.max);

export const LUMA_SOFTNESS = toolRange(0, 1, 0.01, DEFAULT_LUMA_RANGE.softness);

export const COLOR_TOLERANCE = toolRange(0, 1, 0.01, DEFAULT_COLOR_RANGE.tolerance);

export const COLOR_SOFTNESS = toolRange(0, 1, 0.01, DEFAULT_COLOR_RANGE.softness);

export const DEPTH_SOFTNESS = toolRange(0, 1, 0.01, DEFAULT_DEPTH_RANGE.softness);

export const GENERATED_GROW = toolRange(-32, 32, 1, DEFAULT_GENERATED_GROW, { format: whole });

export const GENERATED_FEATHER = toolRange(0, 32, 1, DEFAULT_GENERATED_FEATHER, { format: whole });
