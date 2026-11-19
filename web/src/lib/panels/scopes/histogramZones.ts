import { settleValue } from '#lib/components/editor/controls/sliderStep.ts';
import { blacks, exposure, highlights, shadows, whites } from '#lib/editor/params/basic.ts';
import type { ParamDef, ParamRange } from '#lib/editor/params/types.ts';

export interface HistogramZone {
  param: ParamDef;
  from: number;
  to: number;
}

const SPAN_PER_WIDTH = 0.5;
const WHITES: HistogramZone = { param: whites, from: 0.9, to: 1 };

export const HISTOGRAM_ZONES: readonly HistogramZone[] = [
  { param: blacks, from: 0, to: 0.1 },
  { param: shadows, from: 0.1, to: 0.35 },
  { param: exposure, from: 0.35, to: 0.65 },
  { param: highlights, from: 0.65, to: 0.9 },
  WHITES
];

export function zoneAt(x: number): HistogramZone {
  return HISTOGRAM_ZONES.find((zone) => x < zone.to) ?? WHITES;
}

export function zoneValue(range: ParamRange, start: number, dx: number, width: number): number {
  const span = (range.max - range.min) * SPAN_PER_WIDTH;
  return settleValue(start + (dx / Math.max(width, 1)) * span, range);
}
