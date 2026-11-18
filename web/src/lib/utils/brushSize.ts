import type { ToolRange } from '#lib/editor/params/types.ts';
import { held, type ModifierKeys } from '#lib/input/modifiers.svelte.ts';
import type { PreviewMeta } from '#lib/types/preview.ts';

type BrushRange = Pick<ToolRange, 'min' | 'max' | 'keyStep'>;

type BrushWheelEvent = ModifierKeys &
  Pick<WheelEvent, 'deltaX' | 'deltaY' | 'deltaMode' | 'preventDefault' | 'stopPropagation'>;

type BrushShape = { size: number; hardness: number };

type BrushRanges = { size: BrushRange; hardness: BrushRange };

const WHEEL_NOTCH_PX = 100;
const WHEEL_LINE_PX = 16;
const SIZE_PER_NOTCH = 1.1;

export function formatBrushSize(size: number, meta: PreviewMeta | null): string {
  const shortEdge = meta ? Math.min(meta.source_w, meta.source_h) : 0;
  return shortEdge > 1 ? `${Math.round(size * shortEdge)} px` : size.toFixed(3);
}

function clamp(value: number, { min, max }: BrushRange): number {
  return Math.min(max, Math.max(min, value));
}

export function stepBrush(current: number, key: string, range: BrushRange): number {
  const delta = key === '[' || key === '{' ? -range.keyStep : range.keyStep;
  return clamp(current + delta, range);
}

export function wheelNotches(e: Pick<WheelEvent, 'deltaX' | 'deltaY' | 'deltaMode'>): number {
  const delta = e.deltaY || e.deltaX;
  const px = e.deltaMode === 1 ? delta * WHEEL_LINE_PX : delta;
  return -px / WHEEL_NOTCH_PX;
}

export function wheelSize(current: number, notches: number, range: BrushRange): number {
  return clamp(current * SIZE_PER_NOTCH ** notches, range);
}

export function wheelHardness(current: number, notches: number, range: BrushRange): number {
  return clamp(current + notches * range.keyStep, range);
}

export function brushWheel(
  e: BrushWheelEvent,
  tool: BrushShape,
  ranges: BrushRanges,
  apply: (patch: Partial<BrushShape>) => void
): void {
  if (!held('resize', e)) return;
  e.preventDefault();
  e.stopPropagation();
  const notches = wheelNotches(e);
  if (held('hardness', e)) {
    apply({ hardness: wheelHardness(tool.hardness, notches, ranges.hardness) });
    return;
  }
  apply({ size: wheelSize(tool.size, notches, ranges.size) });
}
