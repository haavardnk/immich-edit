import { neutralEdits, isIdentity, manifestToEdits, FULL_CROP, type AspectLock, type CropRect, type Edits, type MaskComponent, type MaskComponentKind, type MaskComponentMode, type MaskLayer, type MaskedEditKey } from '$lib/types/edits';
import {
  cloneLayerWithNewIds,
  defaultBrush,
  defaultLinear,
  defaultMaskColor,
  makeComponent,
  makeLayer,
  maskCapacity,
  nextLayerName,
  setMaskedEdit
} from '$lib/types/masks';
import { blankBuffer, type BrushBuffer } from '$lib/utils/brush';
import { fetchRaster, uploadRaster } from '$lib/api/rasters';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail, ExifInfo, TagRef } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits } from '$lib/api/edits';
import { ConflictError } from '$lib/api/client';
import type { EditRecord } from '$lib/types/edits';
import { livePreview, persistedPreviewUrl, getPreviewMeta, previewModeIsNone, maskWeightPreview, type PreviewMode } from '$lib/api/preview';
import { downloadExport, EXTENSION_BY_FORMAT, uploadToImmich, type ExportOptions, type ImmichExportOptions } from '$lib/api/export';
import { getAsset, updateAsset } from '$lib/api/assets';
import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
import { browsing } from '$lib/stores/browsing.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import { SingleFlight } from '$lib/utils/single-flight';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { downloadBlob } from '$lib/utils/download';
import { constrainCropRect, largestInscribedRect, refitCropAtAspect, aspectRatioFor } from '$lib/utils/geom';

const LIVE_EDGE = 1600;
const MAX_EDGE = 4096;
const HIRES_DEBOUNCE_MS = 300;
const MAX_HISTORY = 50;

export type EditGroup = 'basic' | 'tone' | 'color' | 'detail' | 'effects' | 'lens';

let clipboard: Edits | null = null;

function computeHiresEdge(zoom: number): number {
  const dpr = typeof window !== 'undefined' ? window.devicePixelRatio : 1;
  const vp = typeof window !== 'undefined' ? Math.max(window.innerWidth, window.innerHeight) : 1600;
  const needed = Math.ceil(vp * dpr * Math.max(1, zoom / 100));
  return Math.min(needed, MAX_EDGE);
}

class EditorStore {
  assetId = $state<string | null>(null);
  asset = $state<AssetDetail | null>(null);
  edits = $state<Edits>(neutralEdits());
  previewUrl = $state<string | null>(null);
  meta = $state<PreviewMeta | null>(null);
  pending = $state(false);
  saving = $state(false);
  savedHash = $state<string>('');
  saveError = $state<string | null>(null);
  exporting = $state(false);
  exportingToImmich = $state(false);
  lastUpload = $state<{ kind: 'success' | 'duplicate' | 'error'; message: string } | null>(null);
  hasEdits = $derived(!isIdentity(this.edits));
  lastWarnings = $state<string[]>([]);
  private lastImmichOpts: ImmichExportOptions | null = null;
  private lastExportOpts: ExportOptions | null = null;
  autoBusy = $state(false);
  error = $state<string | null>(null);
  showingOriginal = $state(false);
  splitMode = $state(false);
  splitPos = $state(0.5);
  originalUrl = $state<string | null>(null);
  cropSession = $state<CropSession | null>(null);

  activeLayerId = $state<string | null>(null);
  activeMaskComponentId = $state<string | null>(null);
  maskOverlayVisible = $state(true);
  maskPreviewLayerId = $state<string | null>(null);
  brushTool = $state<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>({
    size: 0.08,
    hardness: 0.5,
    flow: 0.8,
    mode: 'paint'
  });
  brushBuffers = $state<Record<string, BrushBuffer>>({});

  private history = $state<Edits[]>([]);
  private historyCursor = $state(-1);
  private skipHistory = false;

  private initialised = false;
  private hiresTimer: ReturnType<typeof setTimeout> | null = null;
  private renderedEdge = 0;
  private originalEdge = 0;
  private originalGeomKey = '';

  private flight = new SingleFlight<{ edits: Edits; maxEdge: number; previewMode: PreviewMode }, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      this.pending = true;
      const { blob, metaId } = await livePreview(this.assetId, args.edits, args.maxEdge, args.previewMode, signal);
      return { url: makeObjectUrl(blob), metaId };
    },
    (args, result) => {
      const prev = this.previewUrl;
      this.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.pending = false;
      if (previewModeIsNone(args.previewMode)) {
        this.renderedEdge = args.maxEdge;
        if (result.metaId) void this.loadMeta(result.metaId);
        if (this.splitMode) this.refreshOriginal();
      }
    },
    (err) => {
      this.pending = false;
      this.error = (err as Error).message;
    }
  );

  private originalFlight = new SingleFlight<{ edge: number; geomKey: string }, { url: string }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      const snap = $state.snapshot(this.edits) as Edits;
      const neutral = neutralEdits();
      const edits: Edits = {
        ...snap,
        basic: neutral.basic,
        tone: neutral.tone,
        color: neutral.color,
        detail: neutral.detail,
        effects: neutral.effects,
        masks: []
      };
      const { blob } = await livePreview(this.assetId, edits, args.edge, 'none', signal);
      return { url: makeObjectUrl(blob) };
    },
    (args, result) => {
      const prev = this.originalUrl;
      this.originalUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.originalEdge = args.edge;
      this.originalGeomKey = args.geomKey;
    },
    () => {}
  );

  toggleSplit = (): void => {
    if (this.cropSession) return;
    this.splitMode = !this.splitMode;
    if (this.splitMode) {
      this.refreshOriginal();
    } else {
      this.originalFlight.cancel();
      if (this.originalUrl?.startsWith('blob:')) revoke(this.originalUrl);
      this.originalUrl = null;
      this.originalEdge = 0;
      this.originalGeomKey = '';
    }
  };

  setSplitPos = (p: number): void => {
    this.splitPos = Math.min(1, Math.max(0, p));
  };

  private refreshOriginal(): void {
    if (!this.splitMode || !this.assetId) return;
    const edge = this.renderedEdge || LIVE_EDGE;
    const snap = $state.snapshot(this.edits);
    const geomKey = JSON.stringify({ g: snap.geometry, l: snap.lens, o: snap.output });
    if (this.originalEdge === edge && this.originalGeomKey === geomKey && this.originalUrl) return;
    this.originalFlight.submit({ edge, geomKey });
  }

  async load(id: string): Promise<void> {
    if (this.assetId === id && this.initialised) return;
    this.unload();
    this.assetId = id;
    this.error = null;
    try {
      const [a, s] = await Promise.all([getAsset(id), getEdits(id)]);
      this.asset = a;
      this.edits = manifestToEdits(s.manifest);
      this.savedHash = s.hash;
      this.initialised = true;
      this.pushHistory();
      const hiresEdge = computeHiresEdge(100);
      if (hiresEdge > LIVE_EDGE) {
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: 'none' });
        this.scheduleHires();
      } else {
        this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: hiresEdge, previewMode: 'none' });
      }
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  unload(): void {
    this.flight.cancel();
    this.originalFlight.cancel();
    if (this.hiresTimer) { clearTimeout(this.hiresTimer); this.hiresTimer = null; }
    if (this.cropSession) {
      if (this.cropSession.pinnedUrl) revoke(this.cropSession.pinnedUrl);
      this.cropSession = null;
    }
    if (this.previewUrl?.startsWith('blob:')) revoke(this.previewUrl);
    this.previewUrl = null;
    if (this.originalUrl?.startsWith('blob:')) revoke(this.originalUrl);
    this.originalUrl = null;
    this.originalEdge = 0;
    this.originalGeomKey = '';
    this.splitMode = false;
    this.asset = null;
    this.meta = null;
    this.assetId = null;
    this.initialised = false;
    this.edits = neutralEdits();
    this.history = [];
    this.historyCursor = -1;
    this.showingOriginal = false;
    this.renderedEdge = 0;
    this.activeLayerId = null;
    this.activeMaskComponentId = null;
    this.maskPreviewLayerId = null;
    this.brushBuffers = {};
  }

  private pushHistory(): void {
    const trimmed = this.history.slice(0, this.historyCursor + 1);
    trimmed.push($state.snapshot(this.edits));
    if (trimmed.length > MAX_HISTORY) trimmed.shift();
    this.history = trimmed;
    this.historyCursor = this.history.length - 1;
  }

  get canUndo(): boolean {
    return this.historyCursor > 0;
  }

  get canRedo(): boolean {
    return this.historyCursor < this.history.length - 1;
  }

  undo = (): void => {
    if (!this.canUndo) return;
    this.historyCursor--;
    this.edits = $state.snapshot(this.history[this.historyCursor]) as Edits;
    this.skipHistory = true;
    void this.onCommit();
    this.skipHistory = false;
  };

  redo = (): void => {
    if (!this.canRedo) return;
    this.historyCursor++;
    this.edits = $state.snapshot(this.history[this.historyCursor]) as Edits;
    this.skipHistory = true;
    void this.onCommit();
    this.skipHistory = false;
  };

  loadPersisted(): void {
    if (!this.assetId) return;
    const prev = this.previewUrl;
    this.previewUrl = persistedPreviewUrl(this.assetId, MAX_EDGE) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
  }

  showOriginal(): void {
    if (!this.initialised) return;
    this.flight.submit({ edits: neutralEdits(), maxEdge: this.renderedEdge || LIVE_EDGE, previewMode: 'none' });
  }

  onLive = (): void => {
    if (!this.initialised) return;
    this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: 'none' });
    this.scheduleHires();
  };

  onPreview = (mode: PreviewMode): void => {
    if (!this.initialised) return;
    if (this.hiresTimer) { clearTimeout(this.hiresTimer); this.hiresTimer = null; }
    this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: LIVE_EDGE, previewMode: mode });
  };

  endPreview = (): void => {
    if (!this.initialised) return;
    this.onLive();
  };

  onCommit = async (): Promise<void> => {
    if (!this.initialised || !this.assetId) return;
    if (!this.skipHistory) this.pushHistory();
    this.onLive();
    this.saving = true;
    try {
      if (isIdentity(this.edits)) {
        await deleteEdits(this.assetId);
        this.savedHash = '';
      } else {
        const saved = await putEdits(this.assetId, $state.snapshot(this.edits), this.savedHash);
        this.edits = manifestToEdits(saved.manifest);
        this.savedHash = saved.hash;
      }
      this.saveError = null;
    } catch (e) {
      if (e instanceof ConflictError) {
        const current = e.current as EditRecord | undefined;
        if (current) {
          this.edits = manifestToEdits(current.manifest);
          this.savedHash = current.hash;
        }
        this.saveError = null;
        toasts.push('warn', 'Edits were changed elsewhere. Loaded latest version.');
      } else {
        this.saveError = (e as Error).message;
        this.error = (e as Error).message;
      }
    } finally {
      this.saving = false;
    }
  };

  retrySave = (): Promise<void> => this.onCommit();

  onReset = async (): Promise<void> => {
    if (!this.assetId) return;
    this.edits = { ...neutralEdits(), geometry: this.edits.geometry };
    await this.onCommit();
  };

  copyEdits = (): void => {
    if (isIdentity(this.edits)) return;
    clipboard = $state.snapshot(this.edits) as Edits;
    this.hasClipboard = true;
  };

  pasteEdits = async (): Promise<void> => {
    if (!clipboard || !this.initialised) return;
    const snap = structuredClone(clipboard) as Edits;
    this.edits = { ...snap, geometry: this.edits.geometry, masks: this.edits.masks };
    this.onLive();
    await this.onCommit();
  };

  pasteGroup = async (group: EditGroup): Promise<void> => {
    if (!clipboard || !this.initialised) return;
    const snap = structuredClone(clipboard) as Edits;
    this.edits = { ...this.edits, [group]: snap[group] };
    this.onLive();
    await this.onCommit();
  };

  hasClipboard = $state(false);

  onAutoAdjust = async (): Promise<void> => {
    if (!this.assetId || !this.initialised) return;
    this.autoBusy = true;
    try {
      const suggested = await autoEdits(this.assetId, $state.snapshot(this.edits));
      this.edits = {
        ...this.edits,
        basic: {
          ...this.edits.basic,
          exposure_ev: suggested.basic.exposure_ev,
          brightness: suggested.basic.brightness,
          contrast: suggested.basic.contrast,
          vibrance: suggested.basic.vibrance
        },
        tone: { ...suggested.tone }
      };
      this.onLive();
      await this.onCommit();
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.autoBusy = false;
    }
  };

  onExport = async (opts: ExportOptions): Promise<void> => {
    if (!this.assetId) return;
    this.lastExportOpts = opts;
    this.exporting = true;
    try {
      const blob = await downloadExport(this.assetId, $state.snapshot(this.edits), opts);
      const base = (this.asset?.originalFileName ?? this.assetId).replace(/\.[^.]+$/, '');
      const name = `${base}_edit.${EXTENSION_BY_FORMAT[opts.format]}`;
      downloadBlob(blob, name);
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.exporting = false;
    }
  };

  retryExport = async (): Promise<void> => {
    if (this.lastExportOpts) await this.onExport(this.lastExportOpts);
  };

  onUploadToImmich = async (opts: ImmichExportOptions): Promise<void> => {
    if (!this.assetId) return;
    this.lastImmichOpts = opts;
    this.exportingToImmich = true;
    this.lastUpload = null;
    this.lastWarnings = [];
    try {
      const result = await uploadToImmich(this.assetId, $state.snapshot(this.edits), opts);
      const dup = result.status.toLowerCase() === 'duplicate';
      const msg = dup
        ? `Not uploaded: identical asset already exists in Immich (matched by content hash)`
        : `Uploaded ${result.filename} to Immich`;
      toasts.push(dup ? 'warn' : 'success', msg, 10000);
      this.lastWarnings = result.warnings;
      this.lastUpload = { kind: dup ? 'duplicate' : 'success', message: msg };
      if (opts.stackWithOriginal || opts.favorite) {
        try {
          this.asset = await getAsset(this.assetId);
        } catch {
          /* ignore */
        }
      }
    } catch (e) {
      const m = (e as Error).message;
      this.error = m;
      this.lastUpload = { kind: 'error', message: `Upload failed: ${m}` };
      toasts.push('error', `Upload failed: ${m}`, 10000);
    } finally {
      this.exportingToImmich = false;
    }
  };

  retryUpload = async (): Promise<void> => {
    if (this.lastImmichOpts) await this.onUploadToImmich(this.lastImmichOpts);
  };

  maskCapacityFor = (layerId: string | null): ReturnType<typeof maskCapacity> => {
    return maskCapacity(this.edits, layerId);
  };

  activeLayer = (): MaskLayer | null => {
    if (!this.activeLayerId) return null;
    return this.edits.masks.find((l) => l.id === this.activeLayerId) ?? null;
  };

  setActiveLayer = (id: string | null): void => {
    if (this.activeLayerId !== id) this.activeMaskComponentId = null;
    this.activeLayerId = id;
    if (this.maskPreviewLayerId && this.maskPreviewLayerId !== id) {
      this.endMaskPreview();
    }
  };

  activeMaskComponent = (): MaskComponent | null => {
    const layer = this.activeLayer();
    if (!layer || !this.activeMaskComponentId) return null;
    return layer.components.find((c) => c.id === this.activeMaskComponentId) ?? null;
  };

  setActiveMaskComponent = (id: string | null): void => {
    this.activeMaskComponentId = id;
  };

  setMaskComponentFeather = (
    layerId: string,
    componentId: string,
    feather: number
  ): void => {
    const layer = this.edits.masks.find((l) => l.id === layerId);
    const comp = layer?.components.find((c) => c.id === componentId);
    if (!comp) return;
    const f = Math.max(0, Math.min(1, feather));
    if (comp.kind.kind === 'linear') {
      this.updateMaskComponentKind(layerId, componentId, { ...comp.kind, feather: f }, true);
    } else if (comp.kind.kind === 'radial') {
      this.updateMaskComponentKind(layerId, componentId, { ...comp.kind, feather: f }, true);
    }
  };

  toggleMaskOverlay = (): void => {
    this.maskOverlayVisible = !this.maskOverlayVisible;
  };

  previewMaskWeight = (layerId: string): void => {
    if (!this.initialised) return;
    this.maskPreviewLayerId = layerId;
    this.onPreview(maskWeightPreview(layerId));
  };

  endMaskPreview = (): void => {
    if (!this.maskPreviewLayerId) return;
    this.maskPreviewLayerId = null;
    this.endPreview();
  };

  addMaskLayer = async (kind: MaskComponentKind = defaultLinear()): Promise<string | null> => {
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    const layer = makeLayer(nextLayerName(this.edits.masks), this.edits.masks.length, kind);
    this.edits = { ...this.edits, masks: [...this.edits.masks, layer] };
    this.activeLayerId = layer.id;
    this.activeMaskComponentId = layer.components[0]?.id ?? null;
    await this.onCommit();
    return layer.id;
  };

  removeMaskLayer = async (id: string): Promise<void> => {
    const idx = this.edits.masks.findIndex((l) => l.id === id);
    if (idx < 0) return;
    const masks = this.edits.masks.filter((l) => l.id !== id);
    this.edits = { ...this.edits, masks };
    if (this.activeLayerId === id) {
      this.activeLayerId = masks[idx]?.id ?? masks[masks.length - 1]?.id ?? null;
      this.activeMaskComponentId = null;
    }
    if (this.maskPreviewLayerId === id) this.endMaskPreview();
    await this.onCommit();
  };

  duplicateMaskLayer = async (id: string): Promise<string | null> => {
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    const src = this.edits.masks.find((l) => l.id === id);
    if (!src) return null;
    const copy = cloneLayerWithNewIds(
      src,
      defaultMaskColor(this.edits.masks.length),
      `${src.name} copy`
    );
    const idx = this.edits.masks.findIndex((l) => l.id === id);
    const masks = [...this.edits.masks];
    masks.splice(idx + 1, 0, copy);
    this.edits = { ...this.edits, masks };
    this.activeLayerId = copy.id;
    this.activeMaskComponentId = copy.components[0]?.id ?? null;
    await this.onCommit();
    return copy.id;
  };

  reorderMaskLayer = async (id: string, toIndex: number): Promise<void> => {
    const from = this.edits.masks.findIndex((l) => l.id === id);
    if (from < 0) return;
    const masks = [...this.edits.masks];
    const [layer] = masks.splice(from, 1);
    const clamped = Math.max(0, Math.min(toIndex, masks.length));
    masks.splice(clamped, 0, layer);
    this.edits = { ...this.edits, masks };
    await this.onCommit();
  };

  patchMaskLayer = (id: string, patch: Partial<MaskLayer>, live = true): void => {
    const masks = this.edits.masks.map((l) => (l.id === id ? { ...l, ...patch } : l));
    this.edits = { ...this.edits, masks };
    if (live) this.onLive();
  };

  toggleMaskLayerEnabled = async (id: string): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { enabled: !layer.enabled }, false);
    await this.onCommit();
  };

  renameMaskLayer = async (id: string, name: string): Promise<void> => {
    this.patchMaskLayer(id, { name }, false);
    await this.onCommit();
  };

  setMaskLayerColor = async (id: string, color: string): Promise<void> => {
    this.patchMaskLayer(id, { color }, false);
    await this.onCommit();
  };

  setMaskLayerAmount = (id: string, amount: number): void => {
    this.patchMaskLayer(id, { amount: Math.max(0, Math.min(1, amount)) }, true);
  };

  setMaskLayerEdit = (id: string, key: MaskedEditKey, value: number): void => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { edits: setMaskedEdit(layer.edits, key, value) }, true);
  };

  addMaskComponent = async (
    layerId: string,
    kind: MaskComponentKind,
    mode: MaskComponentMode = 'add'
  ): Promise<string | null> => {
    const cap = maskCapacity(this.edits, layerId);
    if (cap.componentsFull || cap.totalFull) return null;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return null;
    const comp = makeComponent(kind, mode);
    this.patchMaskLayer(layerId, { components: [...layer.components, comp] }, false);
    this.activeMaskComponentId = comp.id;
    await this.onCommit();
    return comp.id;
  };

  removeMaskComponent = async (layerId: string, componentId: string): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return;
    const components = layer.components.filter((c) => c.id !== componentId);
    this.patchMaskLayer(layerId, { components }, false);
    if (this.activeMaskComponentId === componentId) this.activeMaskComponentId = null;
    if (this.brushBuffers[componentId]) {
      const { [componentId]: _drop, ...rest } = this.brushBuffers;
      this.brushBuffers = rest;
    }
    await this.onCommit();
  };

  patchMaskComponent = (
    layerId: string,
    componentId: string,
    patch: Partial<MaskComponent>,
    live = true
  ): void => {
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return;
    const components = layer.components.map((c) =>
      c.id === componentId ? { ...c, ...patch } : c
    );
    this.patchMaskLayer(layerId, { components }, live);
  };

  updateMaskComponentKind = (
    layerId: string,
    componentId: string,
    kind: MaskComponentKind,
    live = true
  ): void => {
    this.patchMaskComponent(layerId, componentId, { kind }, live);
  };

  commitMasks = async (): Promise<void> => {
    await this.onCommit();
  };

  setBrushTool = (
    patch: Partial<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>
  ): void => {
    this.brushTool = { ...this.brushTool, ...patch };
  };

  private brushDims = (): { width: number; height: number } => {
    const sw = this.meta?.source_w ?? 1024;
    const sh = this.meta?.source_h ?? 1024;
    const longest = Math.max(sw, sh, 1);
    const scale = Math.max(1, Math.ceil(longest / 2048));
    return {
      width: Math.max(1, Math.floor(sw / scale)),
      height: Math.max(1, Math.floor(sh / scale))
    };
  };

  ensureBrushBuffer = async (
    componentId: string,
    rasterId: string | null
  ): Promise<BrushBuffer> => {
    const existing = this.brushBuffers[componentId];
    if (existing) return existing;
    if (rasterId) {
      try {
        const r = await fetchRaster(rasterId);
        const buf: BrushBuffer = { width: r.width, height: r.height, bytes: r.bytes };
        this.brushBuffers = { ...this.brushBuffers, [componentId]: buf };
        return buf;
      } catch {
        // fall through to blank
      }
    }
    const { width, height } = this.brushDims();
    const buf = blankBuffer(width, height);
    this.brushBuffers = { ...this.brushBuffers, [componentId]: buf };
    return buf;
  };

  addBrushLayer = async (): Promise<string | null> => {
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    const { width, height } = this.brushDims();
    const buf = blankBuffer(width, height);
    let rasterId: string;
    try {
      const meta = await uploadRaster(width, height, buf.bytes);
      rasterId = meta.raster_id;
    } catch (e) {
      this.error = (e as Error).message;
      return null;
    }
    const layer = makeLayer(
      nextLayerName(this.edits.masks),
      this.edits.masks.length,
      defaultBrush(rasterId)
    );
    this.edits = { ...this.edits, masks: [...this.edits.masks, layer] };
    this.activeLayerId = layer.id;
    const compId = layer.components[0]?.id ?? null;
    this.activeMaskComponentId = compId;
    if (compId) this.brushBuffers = { ...this.brushBuffers, [compId]: buf };
    await this.onCommit();
    return layer.id;
  };

  addBrushComponent = async (layerId: string): Promise<string | null> => {
    const cap = maskCapacity(this.edits, layerId);
    if (cap.componentsFull || cap.totalFull) return null;
    const { width, height } = this.brushDims();
    const buf = blankBuffer(width, height);
    let rasterId: string;
    try {
      const meta = await uploadRaster(width, height, buf.bytes);
      rasterId = meta.raster_id;
    } catch (e) {
      this.error = (e as Error).message;
      return null;
    }
    const id = await this.addMaskComponent(layerId, defaultBrush(rasterId));
    if (id) this.brushBuffers = { ...this.brushBuffers, [id]: buf };
    return id;
  };

  commitBrushStroke = async (layerId: string, componentId: string): Promise<void> => {
    const buf = this.brushBuffers[componentId];
    if (!buf) return;
    try {
      const meta = await uploadRaster(buf.width, buf.height, buf.bytes);
      const layer = this.edits.masks.find((l) => l.id === layerId);
      const comp = layer?.components.find((c) => c.id === componentId);
      if (!comp || comp.kind.kind !== 'brush') return;
      this.updateMaskComponentKind(
        layerId,
        componentId,
        { kind: 'brush', raster_id: meta.raster_id },
        false
      );
      await this.onCommit();
    } catch (e) {
      this.error = (e as Error).message;
    }
  };

  private syncBrowsing(): void {
    if (!this.assetId || !this.asset) return;
    browsing.patch(this.assetId, {
      isFavorite: this.asset.isFavorite,
      exifInfo: this.asset.exifInfo ?? null
    });
  }

  toggleFavorite = async (): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prev = this.asset.isFavorite;
    this.asset = { ...this.asset, isFavorite: !prev };
    try {
      const updated = await updateAsset(this.assetId, { isFavorite: !prev });
      this.asset = updated;
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, isFavorite: prev };
      this.error = (e as Error).message;
    }
  };

  setRating = async (rating: number | null): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prevExif: ExifInfo | null = this.asset.exifInfo;
    const nextExif: ExifInfo = prevExif
      ? { ...prevExif, rating }
      : {
          make: null,
          model: null,
          lensModel: null,
          fNumber: null,
          focalLength: null,
          iso: null,
          exposureTime: null,
          exifImageWidth: null,
          exifImageHeight: null,
          dateTimeOriginal: null,
          rating,
          fileSizeInByte: null
        };
    this.asset = { ...this.asset, exifInfo: nextExif };
    try {
      const updated = await updateAsset(this.assetId, { rating });
      this.asset = updated;
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, exifInfo: prevExif };
      this.error = (e as Error).message;
    }
  };

  addTag = async (tag: TagRef): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (this.asset.tags.some((t) => t.id === tag.id)) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: [...prev, tag] };
    try {
      await addTagToAsset(tag.id, this.assetId);
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  removeTag = async (tagId: string): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: prev.filter((t) => t.id !== tagId) };
    try {
      await removeTagFromAsset(tagId, this.assetId);
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  createAndAddTag = async (value: string): Promise<TagRef | null> => {
    if (!this.assetId || !this.asset) return null;
    try {
      const created = await upsertTags([value]);
      const tag = created[0];
      if (!tag) return null;
      const ref: TagRef = { id: tag.id, name: tag.name, value: tag.value };
      if (!this.asset.tags.some((t) => t.id === ref.id)) {
        const prev = this.asset.tags;
        this.asset = { ...this.asset, tags: [...prev, ref] };
        try {
          await addTagToAsset(ref.id, this.assetId);
        } catch (e) {
          if (this.asset) this.asset = { ...this.asset, tags: prev };
          this.error = (e as Error).message;
          return null;
        }
      }
      return ref;
    } catch (e) {
      this.error = (e as Error).message;
      return null;
    }
  };

  private scheduleHires(zoom = 100): void {
    if (this.hiresTimer) clearTimeout(this.hiresTimer);
    this.hiresTimer = setTimeout(() => {
      this.hiresTimer = null;
      if (!this.initialised) return;
      const edge = computeHiresEdge(zoom);
      if (edge <= this.renderedEdge) return;
      this.flight.submit({ edits: $state.snapshot(this.edits), maxEdge: edge, previewMode: 'none' });
    }, HIRES_DEBOUNCE_MS);
  }

  onZoomChange = (zoom: number): void => {
    if (!this.initialised) return;
    const edge = computeHiresEdge(zoom);
    if (edge <= this.renderedEdge) return;
    this.scheduleHires(zoom);
  };

  private async loadMeta(metaId: string): Promise<void> {
    if (!this.assetId) return;
    try {
      this.meta = await getPreviewMeta(this.assetId, metaId);
    } catch {
      this.meta = null;
    }
  }

  enterCropMode = (): void => {
    if (!this.assetId || !this.initialised || this.cropSession) return;
    const baseEdits = $state.snapshot(this.edits) as Edits;
    this.cropSession = {
      pinnedUrl: null,
      pinnedReady: false,
      srcW: 0,
      srcH: 0,
      baseEdits,
      draftRotate: baseEdits.geometry.rotate,
      draftFlipH: baseEdits.geometry.flip_h,
      draftFlipV: baseEdits.geometry.flip_v,
      draftAngle: baseEdits.geometry.rotate_angle,
      draftCrop: baseEdits.geometry.crop ?? FULL_CROP,
      draftAspect: baseEdits.geometry.aspect,
      userEditedCrop: baseEdits.geometry.crop !== null
    };
    void this.loadPinnedPreview(baseEdits);
  };

  private async loadPinnedPreview(baseEdits: Edits): Promise<void> {
    if (!this.assetId) return;
    const canonical: Edits = {
      ...baseEdits,
      geometry: {
        ...baseEdits.geometry,
        rotate: 0,
        flip_h: false,
        flip_v: false,
        rotate_angle: 0,
        crop: null
      }
    };
    let url: string | null = null;
    try {
      const { blob } = await livePreview(this.assetId, canonical, LIVE_EDGE, 'none');
      url = makeObjectUrl(blob);
      const dims = await new Promise<{ w: number; h: number }>((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight });
        img.onerror = () => reject(new Error('pinned preview decode failed'));
        img.src = url as string;
      });
      const sess = this.cropSession;
      if (!sess || dims.w <= 0 || dims.h <= 0) {
        revoke(url);
        return;
      }
      if (sess.pinnedUrl) revoke(sess.pinnedUrl);
      sess.pinnedUrl = url;
      sess.srcW = dims.w;
      sess.srcH = dims.h;
      sess.pinnedReady = true;
    } catch (e) {
      if (url) revoke(url);
      this.error = (e as Error).message;
    }
  }

  exitCropMode = async (): Promise<void> => {
    const sess = this.cropSession;
    if (!sess) return;
    if (sess.pinnedUrl) revoke(sess.pinnedUrl);
    this.cropSession = null;
    const dc = sess.draftCrop;
    const full = dc.x === 0 && dc.y === 0 && dc.w === 1 && dc.h === 1;
    this.edits = {
      ...this.edits,
      geometry: {
        ...this.edits.geometry,
        rotate: sess.draftRotate,
        flip_h: sess.draftFlipH,
        flip_v: sess.draftFlipV,
        rotate_angle: sess.draftAngle,
        crop: full ? null : sess.draftCrop,
        aspect: sess.draftAspect
      }
    };
    await this.onCommit();
  };

  rotateStep = (delta: 90 | 270): void => {
    const sess = this.cropSession;
    if (sess) {
      sess.draftRotate = ((sess.draftRotate + delta) % 360) as 0 | 90 | 180 | 270;
      const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
      const sw = swapped ? sess.srcH : sess.srcW;
      const sh = swapped ? sess.srcW : sess.srcH;
      const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
      sess.draftCrop = ratio !== null
        ? largestInscribedRect(sw, sh, sess.draftAngle, ratio)
        : FULL_CROP;
      sess.userEditedCrop = false;
      return;
    }
    this.edits.geometry.rotate = ((this.edits.geometry.rotate + delta) % 360) as 0 | 90 | 180 | 270;
    void this.onCommit();
  };

  flipStep = (axis: 'h' | 'v'): void => {
    const sess = this.cropSession;
    if (sess) {
      if (axis === 'h') sess.draftFlipH = !sess.draftFlipH;
      else sess.draftFlipV = !sess.draftFlipV;
      return;
    }
    if (axis === 'h') this.edits.geometry.flip_h = !this.edits.geometry.flip_h;
    else this.edits.geometry.flip_v = !this.edits.geometry.flip_v;
    void this.onCommit();
  };

  updateCropDraftAngle = (angle: number): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftAngle = angle;
    const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = refitCropAtAspect(sess.draftCrop, sw, sh, angle, ratio);
    } else if (!sess.userEditedCrop) {
      sess.draftCrop = largestInscribedRect(sw, sh, angle, sw / sh);
    } else {
      sess.draftCrop = constrainCropRect(sess.draftCrop, sess.draftCrop, sw, sh, angle);
    }
  };

  updateCropDraftCrop = (crop: CropRect): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftCrop = constrainCropRect(crop, sess.draftCrop, sw, sh, sess.draftAngle);
    sess.userEditedCrop = true;
  };

  updateCropDraftAspect = (aspect: AspectLock): void => {
    const sess = this.cropSession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftAspect = aspect;
    const ratio = aspectRatioFor(aspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = largestInscribedRect(sw, sh, sess.draftAngle, ratio);
      sess.userEditedCrop = false;
    }
  };

  resetCropDraft = (): void => {
    const sess = this.cropSession;
    if (!sess) return;
    sess.draftAngle = 0;
    sess.draftAspect = { kind: 'original' };
    sess.draftCrop = FULL_CROP;
    sess.userEditedCrop = false;
  };
}

function sourceDims(sess: CropSession): { sw: number; sh: number } {
  const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
  return { sw: swapped ? sess.srcH : sess.srcW, sh: swapped ? sess.srcW : sess.srcH };
}

interface CropSession {
  pinnedUrl: string | null;
  pinnedReady: boolean;
  srcW: number;
  srcH: number;
  baseEdits: Edits;
  draftRotate: 0 | 90 | 180 | 270;
  draftFlipH: boolean;
  draftFlipV: boolean;
  draftAngle: number;
  draftCrop: CropRect;
  draftAspect: AspectLock;
  userEditedCrop: boolean;
}

export const editor = new EditorStore();
