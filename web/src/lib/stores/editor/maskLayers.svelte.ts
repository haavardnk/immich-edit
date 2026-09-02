import { maskWeightPreview, type PreviewMode } from '$lib/api/preview';
import type { BrushBuffer } from '$lib/utils/brush';
import {
  cloneLayerWithNewIds,
  defaultLinear,
  defaultMaskColor,
  makeComponent,
  makeLayer,
  maskCapacity,
  MAX_POLYGON_POINTS,
  nextLayerName,
  setMaskedEdit
} from '$lib/types/masks';
import type {
  Edits,
  MaskComponent,
  MaskComponentKind,
  MaskComponentMode,
  MaskLayer,
  MaskedEditKey,
  Vec2f
} from '$lib/types/edits';

export interface MaskLayersCtx {
  edits: Edits;
  initialised: boolean;
  activeLayerId: string | null;
  activeMaskComponentId: string | null;
  maskOverlayVisible: boolean;
  maskPreviewLayerId: string | null;
  colorPicker: { layerId: string; componentId: string; ready: boolean } | null;
  brushTool: { size: number; hardness: number; flow: number; mode: 'paint' | 'erase' };
  brushBuffers: Record<string, BrushBuffer>;
  brushBufferSource: Record<string, string>;
  clickTool: {
    active: boolean;
    negative: boolean;
    box: boolean;
    layerId: string | null;
    mode: MaskComponentMode;
  };
  polygonDraft: {
    layerId: string | null;
    mode: MaskComponentMode;
    points: Vec2f[];
  } | null;
  splitMode: boolean;
  clearView(): void;
  toggleSplit(): void;
  onPreview(mode: PreviewMode): void;
  endPreview(): void;
  onLive(): void;
  onCommit(action?: string): Promise<void>;
  submitColorPickerPreview(edits: Edits): void;
}

export function maskCapacityFor(
  ctx: MaskLayersCtx,
  layerId: string | null
): ReturnType<typeof maskCapacity> {
  return maskCapacity(ctx.edits, layerId);
}

export function activeLayer(ctx: MaskLayersCtx): MaskLayer | null {
  if (!ctx.activeLayerId) return null;
  return ctx.edits.masks.find((layer) => layer.id === ctx.activeLayerId) ?? null;
}

export function setActiveLayer(ctx: MaskLayersCtx, id: string | null): void {
  if (ctx.colorPicker && ctx.colorPicker.layerId !== id) cancelColorPicker(ctx);
  if (ctx.activeLayerId !== id) ctx.activeMaskComponentId = null;
  ctx.activeLayerId = id;
  if (ctx.maskPreviewLayerId && ctx.maskPreviewLayerId !== id) endMaskPreview(ctx);
}

export function activeMaskComponent(ctx: MaskLayersCtx): MaskComponent | null {
  const layer = activeLayer(ctx);
  if (!layer || !ctx.activeMaskComponentId) return null;
  return layer.components.find((component) => component.id === ctx.activeMaskComponentId) ?? null;
}

export function setActiveMaskComponent(ctx: MaskLayersCtx, id: string | null): void {
  if (ctx.colorPicker && ctx.colorPicker.componentId !== id) cancelColorPicker(ctx);
  ctx.activeMaskComponentId = id;
}

export function setMaskComponentFeather(
  ctx: MaskLayersCtx,
  layerId: string,
  componentId: string,
  feather: number
): void {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  const component = layer?.components.find((item) => item.id === componentId);
  if (!component) return;
  const clamped = Math.max(0, Math.min(1, feather));
  if (component.kind.kind === 'linear') {
    updateMaskComponentKind(
      ctx,
      layerId,
      componentId,
      { ...component.kind, feather: clamped },
      true
    );
  } else if (component.kind.kind === 'radial') {
    updateMaskComponentKind(
      ctx,
      layerId,
      componentId,
      { ...component.kind, feather: clamped },
      true
    );
  }
}

export function toggleMaskOverlay(ctx: MaskLayersCtx): void {
  ctx.maskOverlayVisible = !ctx.maskOverlayVisible;
}

export function previewMaskWeight(ctx: MaskLayersCtx, layerId: string): void {
  if (!ctx.initialised) return;
  ctx.maskPreviewLayerId = layerId;
  ctx.onPreview(maskWeightPreview(layerId));
}

export function endMaskPreview(ctx: MaskLayersCtx): void {
  if (!ctx.maskPreviewLayerId) return;
  ctx.maskPreviewLayerId = null;
  ctx.endPreview();
}

export function beginColorPicker(ctx: MaskLayersCtx, layerId: string, componentId: string): void {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  const component = layer?.components.find((item) => item.id === componentId);
  if (!component || component.kind.kind !== 'color_range') return;
  ctx.clearView();
  if (ctx.splitMode) ctx.toggleSplit();
  ctx.maskPreviewLayerId = null;
  ctx.colorPicker = { layerId, componentId, ready: false };
  const edits = $state.snapshot(ctx.edits) as Edits;
  ctx.submitColorPickerPreview({ ...edits, masks: [] });
}

export function cancelColorPicker(ctx: MaskLayersCtx): void {
  if (!ctx.colorPicker) return;
  ctx.colorPicker = null;
  ctx.onLive();
}

export async function commitColorSample(
  ctx: MaskLayersCtx,
  sampleRgb: [number, number, number]
): Promise<void> {
  const picker = ctx.colorPicker;
  if (!picker) return;
  const layer = ctx.edits.masks.find((item) => item.id === picker.layerId);
  const component = layer?.components.find((item) => item.id === picker.componentId);
  if (!component || component.kind.kind !== 'color_range') {
    cancelColorPicker(ctx);
    return;
  }
  ctx.colorPicker = null;
  updateMaskComponentKind(
    ctx,
    picker.layerId,
    picker.componentId,
    { ...component.kind, sample_rgb: sampleRgb },
    false
  );
  await commitMasks(ctx);
}

export async function addMaskLayer(
  ctx: MaskLayersCtx,
  kind: MaskComponentKind = defaultLinear()
): Promise<string | null> {
  const capacity = maskCapacity(ctx.edits, null);
  if (capacity.layersFull || capacity.totalFull) return null;
  const layer = makeLayer(nextLayerName(ctx.edits.masks), ctx.edits.masks.length, kind);
  ctx.edits = { ...ctx.edits, masks: [...ctx.edits.masks, layer] };
  ctx.activeLayerId = layer.id;
  ctx.activeMaskComponentId = layer.components[0]?.id ?? null;
  await ctx.onCommit(`Add ${layer.name}`);
  return layer.id;
}

export async function removeMaskLayer(ctx: MaskLayersCtx, id: string): Promise<void> {
  const index = ctx.edits.masks.findIndex((layer) => layer.id === id);
  if (index < 0) return;
  const name = ctx.edits.masks[index].name;
  const masks = ctx.edits.masks.filter((layer) => layer.id !== id);
  ctx.edits = { ...ctx.edits, masks };
  if (ctx.activeLayerId === id) {
    ctx.activeLayerId = masks[index]?.id ?? masks[masks.length - 1]?.id ?? null;
    ctx.activeMaskComponentId = null;
  }
  if (ctx.maskPreviewLayerId === id) endMaskPreview(ctx);
  await ctx.onCommit(`Delete ${name}`);
}

export async function duplicateMaskLayer(ctx: MaskLayersCtx, id: string): Promise<string | null> {
  const capacity = maskCapacity(ctx.edits, null);
  if (capacity.layersFull || capacity.totalFull) return null;
  const source = ctx.edits.masks.find((layer) => layer.id === id);
  if (!source) return null;
  const copy = cloneLayerWithNewIds(
    source,
    defaultMaskColor(ctx.edits.masks.length),
    `${source.name} copy`
  );
  const index = ctx.edits.masks.findIndex((layer) => layer.id === id);
  const masks = [...ctx.edits.masks];
  masks.splice(index + 1, 0, copy);
  ctx.edits = { ...ctx.edits, masks };
  ctx.activeLayerId = copy.id;
  ctx.activeMaskComponentId = copy.components[0]?.id ?? null;
  await ctx.onCommit(`Duplicate ${source.name}`);
  return copy.id;
}

export async function reorderMaskLayer(
  ctx: MaskLayersCtx,
  id: string,
  toIndex: number
): Promise<void> {
  const from = ctx.edits.masks.findIndex((layer) => layer.id === id);
  if (from < 0) return;
  const masks = [...ctx.edits.masks];
  const [layer] = masks.splice(from, 1);
  const clamped = Math.max(0, Math.min(toIndex, masks.length));
  masks.splice(clamped, 0, layer);
  ctx.edits = { ...ctx.edits, masks };
  await ctx.onCommit('Reorder Masks');
}

export async function reorderMaskComponent(
  ctx: MaskLayersCtx,
  layerId: string,
  id: string,
  toIndex: number
): Promise<void> {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  if (!layer) return;
  const from = layer.components.findIndex((component) => component.id === id);
  if (from < 0) return;
  const components = [...layer.components];
  const [component] = components.splice(from, 1);
  const clamped = Math.max(0, Math.min(toIndex, components.length));
  components.splice(clamped, 0, clamped === 0 ? { ...component, mode: 'add' } : component);
  patchMaskLayer(ctx, layerId, { components }, false);
  await ctx.onCommit('Reorder Mask Shapes');
}

export function patchMaskLayer(
  ctx: MaskLayersCtx,
  id: string,
  patch: Partial<MaskLayer>,
  live = true
): void {
  const masks = ctx.edits.masks.map((layer) => (layer.id === id ? { ...layer, ...patch } : layer));
  ctx.edits = { ...ctx.edits, masks };
  if (!live) return;
  if (ctx.maskPreviewLayerId === id) {
    ctx.onPreview(maskWeightPreview(id));
  } else {
    ctx.onLive();
  }
}

export async function toggleMaskLayerEnabled(ctx: MaskLayersCtx, id: string): Promise<void> {
  const layer = ctx.edits.masks.find((item) => item.id === id);
  if (!layer) return;
  patchMaskLayer(ctx, id, { enabled: !layer.enabled }, false);
  await ctx.onCommit(layer.enabled ? `Disable ${layer.name}` : `Enable ${layer.name}`);
}

export async function renameMaskLayer(ctx: MaskLayersCtx, id: string, name: string): Promise<void> {
  patchMaskLayer(ctx, id, { name }, false);
  await ctx.onCommit('Rename Mask');
}

export async function setMaskLayerColor(
  ctx: MaskLayersCtx,
  id: string,
  color: string
): Promise<void> {
  patchMaskLayer(ctx, id, { color }, false);
  await ctx.onCommit('Change Mask Color');
}

export function setMaskLayerAmount(ctx: MaskLayersCtx, id: string, amount: number): void {
  patchMaskLayer(ctx, id, { amount: Math.max(0, Math.min(1, amount)) }, true);
}

export async function toggleMaskLayerInvert(ctx: MaskLayersCtx, id: string): Promise<void> {
  const layer = ctx.edits.masks.find((item) => item.id === id);
  if (!layer) return;
  patchMaskLayer(ctx, id, { invert: !layer.invert }, false);
  await ctx.onCommit(`Invert ${layer.name}`);
}

export function setMaskLayerEdit(
  ctx: MaskLayersCtx,
  id: string,
  key: MaskedEditKey,
  value: number
): void {
  const layer = ctx.edits.masks.find((item) => item.id === id);
  if (!layer) return;
  patchMaskLayer(ctx, id, { edits: setMaskedEdit(layer.edits, key, value) }, true);
}

export async function resetMaskLayerEdits(ctx: MaskLayersCtx, id: string): Promise<void> {
  patchMaskLayer(ctx, id, { amount: 1, edits: {} }, false);
  await ctx.onCommit('Reset Mask Adjustments');
}

export function beginPolygon(
  ctx: MaskLayersCtx,
  layerId: string | null,
  mode: MaskComponentMode = 'add'
): void {
  setActiveMaskComponent(ctx, null);
  ctx.clickTool = { active: false, negative: false, box: false, layerId: null, mode: 'add' };
  ctx.polygonDraft = { layerId, mode, points: [] };
}

export function addPolygonPoint(ctx: MaskLayersCtx, point: Vec2f): void {
  const draft = ctx.polygonDraft;
  if (!draft || draft.points.length >= MAX_POLYGON_POINTS) return;
  ctx.polygonDraft = { ...draft, points: [...draft.points, point] };
}

export function undoPolygonPoint(ctx: MaskLayersCtx): void {
  const draft = ctx.polygonDraft;
  if (!draft || draft.points.length === 0) return;
  ctx.polygonDraft = { ...draft, points: draft.points.slice(0, -1) };
}

export function cancelPolygon(ctx: MaskLayersCtx): void {
  ctx.polygonDraft = null;
}

export async function finishPolygon(ctx: MaskLayersCtx): Promise<void> {
  const draft = ctx.polygonDraft;
  if (!draft || draft.points.length < 3) return;
  ctx.polygonDraft = null;
  const kind: MaskComponentKind = {
    kind: 'polygon',
    points: draft.points,
    feather: 0.05
  };
  if (draft.layerId && ctx.edits.masks.some((layer) => layer.id === draft.layerId)) {
    await addMaskComponent(ctx, draft.layerId, kind, draft.mode);
    return;
  }
  await addMaskLayer(ctx, kind);
}

export async function addMaskComponent(
  ctx: MaskLayersCtx,
  layerId: string,
  kind: MaskComponentKind,
  mode: MaskComponentMode = 'add'
): Promise<string | null> {
  const capacity = maskCapacity(ctx.edits, layerId);
  if (capacity.componentsFull || capacity.totalFull) return null;
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  if (!layer) return null;
  const component = makeComponent(kind, mode);
  patchMaskLayer(ctx, layerId, { components: [...layer.components, component] }, false);
  ctx.activeMaskComponentId = component.id;
  await ctx.onCommit(`Add ${kind.kind.replaceAll('_', ' ')} Shape`);
  return component.id;
}

export async function removeMaskComponent(
  ctx: MaskLayersCtx,
  layerId: string,
  componentId: string
): Promise<void> {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  if (!layer) return;
  const components = layer.components.filter((component) => component.id !== componentId);
  patchMaskLayer(ctx, layerId, { components }, false);
  if (ctx.activeMaskComponentId === componentId) ctx.activeMaskComponentId = null;
  if (ctx.brushBuffers[componentId]) {
    const { [componentId]: _drop, ...rest } = ctx.brushBuffers;
    ctx.brushBuffers = rest;
  }
  delete ctx.brushBufferSource[componentId];
  await ctx.onCommit('Delete Mask Shape');
}

export function patchMaskComponent(
  ctx: MaskLayersCtx,
  layerId: string,
  componentId: string,
  patch: Partial<MaskComponent>,
  live = true
): void {
  const layer = ctx.edits.masks.find((item) => item.id === layerId);
  if (!layer) return;
  const components = layer.components.map((component) =>
    component.id === componentId ? { ...component, ...patch } : component
  );
  patchMaskLayer(ctx, layerId, { components }, live);
}

export function updateMaskComponentKind(
  ctx: MaskLayersCtx,
  layerId: string,
  componentId: string,
  kind: MaskComponentKind,
  live = true
): void {
  patchMaskComponent(ctx, layerId, componentId, { kind }, live);
}

export async function commitMasks(ctx: MaskLayersCtx): Promise<void> {
  await ctx.onCommit('Adjust Mask');
}

export function setBrushTool(
  ctx: MaskLayersCtx,
  patch: Partial<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>
): void {
  ctx.brushTool = { ...ctx.brushTool, ...patch };
}
