export type WatermarkAnchor =
  | 'top_left'
  | 'top'
  | 'top_right'
  | 'left'
  | 'center'
  | 'right'
  | 'bottom_left'
  | 'bottom'
  | 'bottom_right';

export interface ExportWatermark {
  id: string;
  size: number;
  opacity: number;
  anchor: WatermarkAnchor;
  inset: number;
}

export const WATERMARK_ANCHORS: { value: WatermarkAnchor; label: string }[] = [
  { value: 'top_left', label: 'Top left' },
  { value: 'top', label: 'Top' },
  { value: 'top_right', label: 'Top right' },
  { value: 'left', label: 'Left' },
  { value: 'center', label: 'Center' },
  { value: 'right', label: 'Right' },
  { value: 'bottom_left', label: 'Bottom left' },
  { value: 'bottom', label: 'Bottom' },
  { value: 'bottom_right', label: 'Bottom right' }
];

export const WATERMARK_PERCENT = {
  size: { min: 1, max: 100, default: 20 },
  opacity: { min: 1, max: 100, default: 80 },
  inset: { min: 0, max: 50, default: 3 }
};

export function pickPercent(
  value: unknown,
  range: { min: number; max: number },
  fallback: number
): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return fallback;
  return Math.min(range.max, Math.max(range.min, Math.round(value)));
}
