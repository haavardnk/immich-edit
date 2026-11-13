import { colorWindow, lumaWindow, type RangeSample } from '#lib/editor/rangeSample.ts';
import type { MaskComponentKind } from '#lib/types/edits.ts';
import type { Rgb } from '#lib/utils/oklab.ts';
import {
  cancelRangePicker,
  commitMasks,
  updateMaskComponentKind,
  type MaskLayersCtx
} from './maskLayers.svelte.ts';

export type RangePicker = {
  layerId: string;
  componentId: string;
  ready: boolean;
  samples: RangeSample[];
  tolerance: number;
  preview: Rgb | null;
};

export type RangeSampleMode = { add: boolean; keep: boolean };

export function rangePickerHidesOverlay(picker: RangePicker | null): boolean {
  return picker !== null && picker.samples.length === 0;
}

function rangeComponent(ctx: MaskLayersCtx, layerId: string, componentId: string) {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  const component = layer?.components.find((item) => item.id === componentId);
  const kind = component?.kind.kind;
  return component && (kind === 'luma_range' || kind === 'color_range') ? component : null;
}

export function beginRangePicker(ctx: MaskLayersCtx, layerId: string, componentId: string): void {
  const component = rangeComponent(ctx, layerId, componentId);
  if (!component) return;
  ctx.compare.end();
  const tolerance = component.kind.kind === 'color_range' ? component.kind.tolerance : 0;
  ctx.rangePicker = { layerId, componentId, ready: false, samples: [], tolerance, preview: null };
  ctx.onLive();
}

export function previewRangeSample(ctx: MaskLayersCtx, rgb: Rgb | null): void {
  if (ctx.rangePicker) ctx.rangePicker = { ...ctx.rangePicker, preview: rgb };
}

export function clearRangeSamples(ctx: MaskLayersCtx): void {
  if (ctx.rangePicker) ctx.rangePicker = { ...ctx.rangePicker, samples: [], preview: null };
}

function sampledKind(
  kind: MaskComponentKind,
  samples: readonly RangeSample[],
  tolerance: number
): MaskComponentKind | null {
  if (kind.kind === 'luma_range') {
    const window = lumaWindow(samples);
    return window ? { ...kind, ...window } : null;
  }
  if (kind.kind === 'color_range') {
    const window = colorWindow(samples, tolerance);
    return window ? { ...kind, ...window } : null;
  }
  return null;
}

export async function commitRangeSample(
  ctx: MaskLayersCtx,
  sample: RangeSample,
  { add, keep }: RangeSampleMode
): Promise<void> {
  const picker = ctx.rangePicker;
  if (!picker) return;
  const component = rangeComponent(ctx, picker.layerId, picker.componentId);
  const samples = add ? [...picker.samples, sample] : [sample];
  const kind = component ? sampledKind(component.kind, samples, picker.tolerance) : null;
  if (!component || !kind) {
    cancelRangePicker(ctx);
    return;
  }
  ctx.rangePicker = keep ? { ...picker, samples, preview: null } : null;
  ctx.maskOverlayVisible = true;
  updateMaskComponentKind(ctx, picker.layerId, picker.componentId, kind, false);
  await commitMasks(ctx);
}
