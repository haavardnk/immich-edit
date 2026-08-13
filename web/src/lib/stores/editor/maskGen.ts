import {
  clickMask,
  generateMask,
  rebakeMask,
  type ClickPoint,
  type MaskBox,
  type MaskKind,
  type MaskRange
} from '$lib/api/masks';
import { fetchRaster, uploadRaster } from '$lib/api/rasters';
import { blankBuffer, type BrushBuffer } from '$lib/utils/brush';
import { errorMessage } from '$lib/utils/errors';
import {
  defaultBrush,
  makeComponent,
  makeGeneratedLayer,
  makeLayer,
  maskCapacity,
  nextLayerName
} from '$lib/types/masks';
import type { Edits, MaskComponent, MaskComponentMode, MaskLayer } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';

export interface MaskGenCtx {
  assetId: string | null;
  edits: Edits;
  meta: PreviewMeta | null;
  error: string | null;
  maskGenerating: boolean;
  maskError: string | null;
  maskRetry: (() => Promise<unknown>) | null;
  activeLayerId: string | null;
  activeMaskComponentId: string | null;
  brushBuffers: Record<string, BrushBuffer>;
  brushBufferSource: Record<string, string>;
  clickTool: { layerId: string | null; mode: MaskComponentMode; negative: boolean };
  patchMaskLayer(id: string, patch: Partial<MaskLayer>, live?: boolean): void;
  patchMaskComponent(
    layerId: string,
    componentId: string,
    patch: Partial<MaskComponent>,
    live?: boolean
  ): void;
  updateMaskComponentKind(
    layerId: string,
    componentId: string,
    kind: MaskComponent['kind'],
    live?: boolean
  ): void;
  addMaskComponent(
    layerId: string,
    kind: MaskComponent['kind'],
    mode?: MaskComponentMode
  ): Promise<string | null>;
  removeMaskComponent(layerId: string, componentId: string): Promise<void>;
  onCommit(action?: string): Promise<void>;
}

function fail(ctx: MaskGenCtx, e: unknown, retry: () => Promise<unknown>): void {
  ctx.maskError = errorMessage(e);
  ctx.maskRetry = retry;
}

function layerOf(ctx: MaskGenCtx, layerId: string | null): MaskLayer | null {
  if (!layerId) return null;
  return ctx.edits.masks.find((l) => l.id === layerId) ?? null;
}

function componentOf(layer: MaskLayer | null, componentId: string | null): MaskComponent | null {
  if (!layer || !componentId) return null;
  return layer.components.find((c) => c.id === componentId) ?? null;
}

function activePair(ctx: MaskGenCtx): { layer: MaskLayer | null; comp: MaskComponent | null } {
  const layer = layerOf(ctx, ctx.activeLayerId);
  return { layer, comp: componentOf(layer, ctx.activeMaskComponentId) };
}

export function brushDims(ctx: MaskGenCtx): { width: number; height: number } {
  const sw = ctx.meta?.source_w ?? 1024;
  const sh = ctx.meta?.source_h ?? 1024;
  const longest = Math.max(sw, sh, 1);
  const scale = Math.max(1, Math.ceil(longest / 2048));
  return {
    width: Math.max(1, Math.floor(sw / scale)),
    height: Math.max(1, Math.floor(sh / scale))
  };
}

function storeBuffer(ctx: MaskGenCtx, componentId: string, buf: BrushBuffer, key: string): void {
  ctx.brushBuffers = { ...ctx.brushBuffers, [componentId]: buf };
  ctx.brushBufferSource[componentId] = key;
}

export async function ensureBrushBuffer(
  ctx: MaskGenCtx,
  componentId: string,
  rasterId: string | null
): Promise<BrushBuffer> {
  const key = rasterId ?? '';
  const existing = ctx.brushBuffers[componentId];
  if (existing && ctx.brushBufferSource[componentId] === key) return existing;
  if (rasterId) {
    try {
      const r = await fetchRaster(rasterId);
      const buf: BrushBuffer = { width: r.width, height: r.height, bytes: r.bytes };
      storeBuffer(ctx, componentId, buf, key);
      return buf;
    } catch {
      // fall through to a blank buffer
    }
  }
  const { width, height } = brushDims(ctx);
  const buf = blankBuffer(width, height);
  storeBuffer(ctx, componentId, buf, key);
  return buf;
}

async function newRaster(ctx: MaskGenCtx): Promise<{ id: string; buf: BrushBuffer } | null> {
  const { width, height } = brushDims(ctx);
  const buf = blankBuffer(width, height);
  try {
    const meta = await uploadRaster(width, height, buf.bytes);
    return { id: meta.raster_id, buf };
  } catch (e) {
    ctx.error = errorMessage(e);
    return null;
  }
}

function appendComponent(ctx: MaskGenCtx, layer: MaskLayer, comp: MaskComponent): void {
  ctx.patchMaskLayer(layer.id, { components: [...layer.components, comp] }, false);
  ctx.activeMaskComponentId = comp.id;
}

function appendLayer(ctx: MaskGenCtx, layer: MaskLayer): string | null {
  ctx.edits = { ...ctx.edits, masks: [...ctx.edits.masks, layer] };
  ctx.activeLayerId = layer.id;
  const compId = layer.components[0]?.id ?? null;
  ctx.activeMaskComponentId = compId;
  return compId;
}

export async function addGeneratedComponent(
  ctx: MaskGenCtx,
  layerId: string,
  kind: MaskKind,
  mode: MaskComponentMode = 'add',
  maskClass?: string,
  invert = false
): Promise<string | null> {
  const assetId = ctx.assetId;
  if (!assetId || ctx.maskGenerating) return null;
  const cap = maskCapacity(ctx.edits, layerId);
  if (cap.componentsFull || cap.totalFull) return null;
  const layer = layerOf(ctx, layerId);
  if (!layer) return null;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await generateMask(assetId, kind, 0, 0, maskClass);
    const comp: MaskComponent = {
      ...makeComponent({ kind: 'brush', raster_id: res.raster_id }, mode),
      invert,
      source: 'generated',
      generated: {
        model_id: res.model_id,
        kind,
        prob_raster_id: res.prob_raster_id,
        grow: 0,
        feather: 0,
        ...(maskClass ? { class: maskClass } : {})
      }
    };
    appendComponent(ctx, layer, comp);
    await ensureBrushBuffer(ctx, comp.id, res.raster_id);
    await ctx.onCommit('Masks');
    return comp.id;
  } catch (e) {
    fail(ctx, e, () => addGeneratedComponent(ctx, layerId, kind, mode, maskClass, invert));
    return null;
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function addGeneratedLayer(
  ctx: MaskGenCtx,
  kind: MaskKind,
  maskClass?: string,
  invert = false
): Promise<string | null> {
  const assetId = ctx.assetId;
  if (!assetId || ctx.maskGenerating) return null;
  const cap = maskCapacity(ctx.edits, null);
  if (cap.layersFull || cap.totalFull) return null;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await generateMask(assetId, kind, 0, 0, maskClass);
    const layer = makeGeneratedLayer(
      nextLayerName(ctx.edits.masks),
      ctx.edits.masks.length,
      res.raster_id,
      {
        model_id: res.model_id,
        kind,
        prob_raster_id: res.prob_raster_id,
        grow: 0,
        feather: 0,
        ...(maskClass ? { class: maskClass } : {})
      },
      invert
    );
    const compId = appendLayer(ctx, layer);
    if (compId) await ensureBrushBuffer(ctx, compId, res.raster_id);
    await ctx.onCommit('Masks');
    return layer.id;
  } catch (e) {
    fail(ctx, e, () => addGeneratedLayer(ctx, kind, maskClass, invert));
    return null;
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function rebakeGeneratedComponent(
  ctx: MaskGenCtx,
  layerId: string,
  componentId: string,
  grow: number,
  feather: number,
  range?: MaskRange
): Promise<void> {
  const assetId = ctx.assetId;
  const comp = componentOf(layerOf(ctx, layerId), componentId);
  if (!assetId || !comp?.generated || ctx.maskGenerating) return;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await rebakeMask(assetId, comp.generated.prob_raster_id, grow, feather, range);
    await ensureBrushBuffer(ctx, componentId, res.raster_id);
    ctx.patchMaskComponent(
      layerId,
      componentId,
      {
        kind: { kind: 'brush', raster_id: res.raster_id },
        generated: {
          ...comp.generated,
          grow,
          feather,
          painted: false,
          ...(range ? { range } : {})
        }
      },
      false
    );
    await ctx.onCommit('Masks');
  } catch (e) {
    fail(ctx, e, () => rebakeGeneratedComponent(ctx, layerId, componentId, grow, feather, range));
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function clickRefineComponent(
  ctx: MaskGenCtx,
  layerId: string,
  componentId: string,
  points: ClickPoint[]
): Promise<void> {
  const assetId = ctx.assetId;
  const comp = componentOf(layerOf(ctx, layerId), componentId);
  if (!assetId || !comp?.generated || ctx.maskGenerating) return;
  if (points.length === 0) return;
  const grow = comp.generated.grow;
  const feather = comp.generated.feather;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await clickMask(assetId, points, grow, feather);
    await ensureBrushBuffer(ctx, componentId, res.raster_id);
    ctx.patchMaskComponent(
      layerId,
      componentId,
      {
        kind: { kind: 'brush', raster_id: res.raster_id },
        generated: {
          ...comp.generated,
          model_id: res.model_id,
          prob_raster_id: res.prob_raster_id,
          painted: false,
          points
        }
      },
      false
    );
    await ctx.onCommit('Masks');
  } catch (e) {
    fail(ctx, e, () => clickRefineComponent(ctx, layerId, componentId, points));
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function addClickLayer(
  ctx: MaskGenCtx,
  points: ClickPoint[],
  bbox?: MaskBox
): Promise<string | null> {
  const assetId = ctx.assetId;
  if (!assetId || ctx.maskGenerating || (points.length === 0 && !bbox)) return null;
  const cap = maskCapacity(ctx.edits, null);
  if (cap.layersFull || cap.totalFull) return null;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await clickMask(assetId, points, 0, 0, undefined, false, bbox);
    const layer = makeGeneratedLayer(
      nextLayerName(ctx.edits.masks),
      ctx.edits.masks.length,
      res.raster_id,
      {
        model_id: res.model_id,
        kind: 'click',
        prob_raster_id: res.prob_raster_id,
        grow: 0,
        feather: 0,
        points
      }
    );
    const compId = appendLayer(ctx, layer);
    if (compId) await ensureBrushBuffer(ctx, compId, res.raster_id);
    await ctx.onCommit('Masks');
    return layer.id;
  } catch (e) {
    fail(ctx, e, () => addClickLayer(ctx, points, bbox));
    return null;
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function addClickComponent(
  ctx: MaskGenCtx,
  layerId: string,
  points: ClickPoint[],
  mode: MaskComponentMode = 'add',
  bbox?: MaskBox
): Promise<string | null> {
  const assetId = ctx.assetId;
  if (!assetId || ctx.maskGenerating || (points.length === 0 && !bbox)) return null;
  const cap = maskCapacity(ctx.edits, layerId);
  if (cap.componentsFull || cap.totalFull) return null;
  const layer = layerOf(ctx, layerId);
  if (!layer) return null;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await clickMask(assetId, points, 0, 0, undefined, false, bbox);
    const comp: MaskComponent = {
      ...makeComponent({ kind: 'brush', raster_id: res.raster_id }, mode),
      source: 'generated',
      generated: {
        model_id: res.model_id,
        kind: 'click',
        prob_raster_id: res.prob_raster_id,
        grow: 0,
        feather: 0,
        points
      }
    };
    appendComponent(ctx, layer, comp);
    await ensureBrushBuffer(ctx, comp.id, res.raster_id);
    await ctx.onCommit('Masks');
    return comp.id;
  } catch (e) {
    fail(ctx, e, () => addClickComponent(ctx, layerId, points, mode, bbox));
    return null;
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function clickRefineRaster(
  ctx: MaskGenCtx,
  layerId: string,
  componentId: string,
  points: ClickPoint[],
  subtract: boolean,
  bbox?: MaskBox
): Promise<void> {
  const assetId = ctx.assetId;
  const comp = componentOf(layerOf(ctx, layerId), componentId);
  if (!assetId || !comp || comp.kind.kind !== 'brush' || ctx.maskGenerating) return;
  const base = comp.generated?.prob_raster_id ?? comp.kind.raster_id;
  const grow = comp.generated?.grow ?? 0;
  const feather = comp.generated?.feather ?? 0;
  ctx.maskGenerating = true;
  ctx.maskError = null;
  try {
    const res = await clickMask(
      assetId,
      points.map((p) => ({ ...p, positive: true })),
      grow,
      feather,
      base,
      subtract,
      bbox
    );
    await ensureBrushBuffer(ctx, componentId, res.raster_id);
    ctx.patchMaskComponent(
      layerId,
      componentId,
      {
        kind: { kind: 'brush', raster_id: res.raster_id },
        ...(comp.generated
          ? {
              generated: {
                ...comp.generated,
                prob_raster_id: res.prob_raster_id,
                points: []
              }
            }
          : {})
      },
      false
    );
    await ctx.onCommit('Masks');
  } catch (e) {
    fail(ctx, e, () => clickRefineRaster(ctx, layerId, componentId, points, subtract, bbox));
  } finally {
    ctx.maskGenerating = false;
  }
}

export async function removeClickPoint(ctx: MaskGenCtx, index: number): Promise<void> {
  if (ctx.maskGenerating) return;
  const { layer, comp } = activePair(ctx);
  if (!layer || !comp || comp.generated?.kind !== 'click') return;
  const points = comp.generated.points ?? [];
  if (index < 0 || index >= points.length) return;
  const next = points.filter((_, i) => i !== index);
  if (next.length === 0) {
    await ctx.removeMaskComponent(layer.id, comp.id);
    return;
  }
  await clickRefineComponent(ctx, layer.id, comp.id, next);
}

export async function addClickPoint(
  ctx: MaskGenCtx,
  x: number,
  y: number,
  positive: boolean
): Promise<void> {
  if (ctx.maskGenerating) return;
  const { layer, comp } = activePair(ctx);
  const point = { x, y, positive };
  if (layer && comp?.generated?.kind === 'click' && (comp.generated.points ?? []).length > 0) {
    await clickRefineComponent(ctx, layer.id, comp.id, [...(comp.generated.points ?? []), point]);
    return;
  }
  if (layer && comp && comp.kind.kind === 'brush') {
    await clickRefineRaster(ctx, layer.id, comp.id, [point], !positive);
    return;
  }
  if (!positive) return;
  const target = ctx.clickTool.layerId;
  if (target && ctx.edits.masks.some((l) => l.id === target)) {
    await addClickComponent(ctx, target, [point], ctx.clickTool.mode);
    return;
  }
  await addClickLayer(ctx, [point]);
}

export async function addClickBox(ctx: MaskGenCtx, bbox: MaskBox): Promise<void> {
  if (ctx.maskGenerating) return;
  const { layer, comp } = activePair(ctx);
  if (layer && comp && comp.kind.kind === 'brush') {
    await clickRefineRaster(ctx, layer.id, comp.id, [], ctx.clickTool.negative, bbox);
    return;
  }
  const target = ctx.clickTool.layerId;
  if (target && ctx.edits.masks.some((l) => l.id === target)) {
    await addClickComponent(ctx, target, [], ctx.clickTool.mode, bbox);
    return;
  }
  await addClickLayer(ctx, [], bbox);
}

export async function addBrushLayer(ctx: MaskGenCtx): Promise<string | null> {
  const cap = maskCapacity(ctx.edits, null);
  if (cap.layersFull || cap.totalFull) return null;
  const raster = await newRaster(ctx);
  if (!raster) return null;
  const layer = makeLayer(
    nextLayerName(ctx.edits.masks),
    ctx.edits.masks.length,
    defaultBrush(raster.id)
  );
  const compId = appendLayer(ctx, layer);
  if (compId) ctx.brushBuffers = { ...ctx.brushBuffers, [compId]: raster.buf };
  await ctx.onCommit('Masks');
  return layer.id;
}

export async function addBrushComponent(
  ctx: MaskGenCtx,
  layerId: string,
  mode: MaskComponentMode = 'add'
): Promise<string | null> {
  const cap = maskCapacity(ctx.edits, layerId);
  if (cap.componentsFull || cap.totalFull) return null;
  const raster = await newRaster(ctx);
  if (!raster) return null;
  const id = await ctx.addMaskComponent(layerId, defaultBrush(raster.id), mode);
  if (id) ctx.brushBuffers = { ...ctx.brushBuffers, [id]: raster.buf };
  return id;
}

export async function commitBrushStroke(
  ctx: MaskGenCtx,
  layerId: string,
  componentId: string
): Promise<void> {
  const buf = ctx.brushBuffers[componentId];
  if (!buf) return;
  try {
    const meta = await uploadRaster(buf.width, buf.height, buf.bytes);
    const comp = componentOf(layerOf(ctx, layerId), componentId);
    if (!comp || comp.kind.kind !== 'brush') return;
    ctx.brushBufferSource[componentId] = meta.raster_id;
    ctx.updateMaskComponentKind(
      layerId,
      componentId,
      { kind: 'brush', raster_id: meta.raster_id },
      false
    );
    if (comp.generated && !comp.generated.painted) {
      ctx.patchMaskComponent(
        layerId,
        componentId,
        { generated: { ...comp.generated, painted: true } },
        false
      );
    }
    await ctx.onCommit('Masks');
  } catch (e) {
    ctx.error = errorMessage(e);
  }
}
