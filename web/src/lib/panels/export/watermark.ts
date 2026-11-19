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

type Align = 'start' | 'center' | 'end';

export interface WatermarkBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

function alignOf(anchor: WatermarkAnchor): { x: Align; y: Align } {
  const x = anchor.endsWith('left') ? 'start' : anchor.endsWith('right') ? 'end' : 'center';
  const y = anchor.startsWith('top') ? 'start' : anchor.startsWith('bottom') ? 'end' : 'center';
  return { x, y };
}

function offset(align: Align, outer: number, inner: number, inset: number): number {
  const room = Math.max(0, outer - inner);
  if (align === 'start') return Math.min(inset, room);
  if (align === 'center') return room / 2;
  return Math.max(0, room - inset);
}

export function watermarkBox(
  photo: { w: number; h: number },
  mark: { width: number; height: number },
  placement: Pick<ExportWatermark, 'size' | 'inset' | 'anchor'>
): WatermarkBox {
  const short = Math.min(photo.w, photo.h);
  const edge = Math.min(short, Math.max(1, (short * placement.size) / 100));
  const scale = edge / Math.max(mark.width, mark.height, 1);
  const width = mark.width * scale;
  const height = mark.height * scale;
  const inset = (short * placement.inset) / 100;
  const align = alignOf(placement.anchor);
  return {
    left: offset(align.x, photo.w, width, inset),
    top: offset(align.y, photo.h, height, inset),
    width,
    height
  };
}
