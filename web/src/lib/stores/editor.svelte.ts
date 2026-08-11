import {
  neutralEdits,
  originalPreviewEdits,
  resetDevelopEdits,
  isIdentity,
  manifestToEdits,
  FULL_CROP,
  MAX_RETOUCH_STROKES,
  type AspectLock,
  type CropRect,
  type Edits,
  type EditManifest,
  type MaskComponent,
  type MaskComponentKind,
  type MaskComponentMode,
  type MaskLayer,
  type MaskedEditKey,
  type RetouchMode,
  type RetouchStroke,
  type Vec2f
} from '$lib/types/edits';
import {
  cloneLayerWithNewIds,
  defaultBrush,
  defaultLinear,
  defaultMaskColor,
  makeComponent,
  MAX_POLYGON_POINTS,
  makeGeneratedLayer,
  makeLayer,
  maskCapacity,
  nextLayerName,
  setMaskedEdit
} from '$lib/types/masks';
import { blankBuffer, type BrushBuffer } from '$lib/utils/brush';
import { fetchRaster, uploadRaster } from '$lib/api/rasters';
import {
  clickMask,
  generateMask,
  rebakeMask,
  type ClickPoint,
  type MaskBox,
  type MaskKind,
  type MaskRange
} from '$lib/api/masks';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail, ExifInfo, TagRef } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits } from '$lib/api/edits';
import { ConflictError, ApiError } from '$lib/api/client';
import type { EditRecord } from '$lib/types/edits';
import {
  livePreview,
  persistedPreviewUrl,
  getPreviewMeta,
  previewModeIsNone,
  maskWeightPreview,
  type PreviewMode,
  type ProofOptions
} from '$lib/api/preview';
import {
  downloadExport,
  EXTENSION_BY_FORMAT,
  uploadToImmich,
  type ColorSpaceOpt,
  type ExportOptions,
  type ImmichExportOptions
} from '$lib/api/export';
import { getAsset, updateAsset } from '$lib/api/assets';
import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
import { browsing } from '$lib/stores/browsing.svelte';
import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
import { rejected } from '$lib/stores/rejected.svelte';
import { ensureRejectTag, isRejected, setRejectedTags } from '$lib/reject';
import { clipboard } from '$lib/stores/clipboard.svelte';
import { copyDialog } from '$lib/stores/copyDialog.svelte';
import { ui } from '$lib/stores/ui.svelte';
import {
  isFullFrame,
  renderRequest,
  visibleRegion,
  type Rect,
  type RenderRequest,
  type Roi
} from '$lib/utils/view-geometry';
import { displayGamutIsWide, previewColorSpace } from '$lib/utils/color-gamut';
import { applyCopySections } from '$lib/copyPaste';
import { toasts } from '$lib/stores/toasts.svelte';
import { SingleFlight } from '$lib/utils/single-flight';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { downloadBlob } from '$lib/utils/download';
import {
  constrainCropRect,
  largestInscribedRect,
  refitCropAtAspect,
  aspectRatioFor
} from '$lib/utils/geom';
import {
  clampPerspective,
  limitPerspective,
  neutralPerspective,
  perspectiveInverse,
  perspectiveIsIdentity,
  type Mat3,
  type PerspectiveEdits
} from '$lib/utils/perspective';

const LIVE_EDGE = 1600;
const MAX_EDGE = 4096;
const VIEW_MAX_EDGE = 8192;
const VIEW_DEBOUNCE_MS = 150;
const MAX_HISTORY = 50;

type ViewSnapshot = { frame: Rect; viewW: number; viewH: number; dpr: number };

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
  private lastSaveAction: string | undefined = undefined;
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
  proofSpace = $state<ColorSpaceOpt>('srgb');
  gamutWarn = $state(false);
  isProofing = $derived(this.proofSpace !== 'srgb' || this.gamutWarn);
  originalUrl = $state<string | null>(null);
  viewUrl = $state<string | null>(null);
  viewRoi = $state<Roi | null>(null);
  viewNat = $state<{ w: number; h: number } | null>(null);
  geometrySession = $state<GeometrySession | null>(null);
  private geometrySessionId = 0;

  activeLayerId = $state<string | null>(null);
  activeMaskComponentId = $state<string | null>(null);
  maskGenerating = $state(false);
  maskError = $state<string | null>(null);
  private maskRetry: (() => Promise<unknown>) | null = null;
  maskOverlayVisible = $state(true);
  maskPreviewLayerId = $state<string | null>(null);
  colorPicker = $state<{ layerId: string; componentId: string; ready: boolean } | null>(null);
  brushTool = $state<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>({
    size: 0.08,
    hardness: 0.5,
    flow: 0.8,
    mode: 'paint'
  });
  brushBuffers = $state<Record<string, BrushBuffer>>({});
  private brushBufferSource: Record<string, string> = {};
  clickTool = $state<{
    active: boolean;
    negative: boolean;
    box: boolean;
    layerId: string | null;
    mode: MaskComponentMode;
  }>({
    active: false,
    negative: false,
    box: false,
    layerId: null,
    mode: 'add'
  });
  polygonDraft = $state<{
    layerId: string | null;
    mode: MaskComponentMode;
    points: Vec2f[];
  } | null>(null);

  retouchTool = $state<{
    mode: RetouchMode;
    size: number;
    hardness: number;
    opacity: number;
  }>({
    mode: 'heal',
    size: 0.05,
    hardness: 0.5,
    opacity: 1
  });
  activeRetouchId = $state<string | null>(null);
  retouchAnchor = $state<Vec2f | null>(null);

  private history = $state<Edits[]>([]);
  private historyCursor = $state(-1);
  private skipHistory = false;

  initialised = $state(false);
  private idleTimer: ReturnType<typeof setTimeout> | null = null;
  private viewSnap: ViewSnapshot | null = null;
  private viewKey = '';
  private viewFullEdge = 0;
  private srcLong = Number.POSITIVE_INFINITY;
  private originalEdge = 0;
  private originalGeomKey = '';

  private flight = new SingleFlight<
    {
      edits: Edits;
      maxEdge: number;
      previewMode: PreviewMode;
      purpose?: 'color-picker';
    },
    { url: string; metaId: string | null }
  >(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      this.pending = true;
      const { blob, metaId } = await livePreview(
        this.assetId,
        args.edits,
        args.maxEdge,
        args.previewMode,
        this.proofOptions(),
        signal
      );
      return { url: makeObjectUrl(blob), metaId };
    },
    (args, result) => {
      const prev = this.previewUrl;
      this.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.pending = false;
      if (args.purpose === 'color-picker' && this.colorPicker) {
        this.colorPicker = { ...this.colorPicker, ready: true };
      }
      if (previewModeIsNone(args.previewMode)) {
        if (result.metaId) void this.loadMeta(result.metaId);
        if (this.splitMode) this.refreshOriginal();
      }
    },
    (err) => {
      this.pending = false;
      if (err instanceof DOMException && err.name === 'AbortError') return;
      if (err instanceof ApiError && err.code === 'superseded') return;
      this.colorPicker = null;
      this.error = (err as Error).message;
    }
  );

  private originalFlight = new SingleFlight<{ edge: number; geomKey: string }, { url: string }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      const snap = $state.snapshot(this.edits) as Edits;
      const edits = originalPreviewEdits(snap);
      const { blob } = await livePreview(
        this.assetId,
        edits,
        args.edge,
        'none',
        this.proofOptions(),
        signal,
        'original'
      );
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

  private viewFlight = new SingleFlight<RenderRequest, { url: string; w: number; h: number }>(
    async (args, signal) => {
      if (!this.assetId) throw new Error('no asset');
      const { blob } = await livePreview(
        this.assetId,
        $state.snapshot(this.edits) as Edits,
        args.maxEdge,
        'none',
        this.proofOptions(),
        signal,
        'roi',
        isFullFrame(args.roi) ? undefined : args.roi
      );
      const url = makeObjectUrl(blob);
      const decoded = new Image();
      decoded.src = url;
      await decoded.decode().catch(() => undefined);
      return { url, w: decoded.naturalWidth, h: decoded.naturalHeight };
    },
    (args, result) => {
      const prev = this.viewUrl;
      this.viewUrl = result.url;
      this.viewRoi = args.roi;
      this.viewNat = { w: result.w, h: result.h };
      const delivered = Math.max(result.w, result.h);
      const fraction = args.fullEdge > 0 ? args.maxEdge / args.fullEdge : 1;
      this.viewFullEdge = Math.round(delivered / fraction);
      if (delivered < args.maxEdge - 1) this.srcLong = this.viewFullEdge;
      if (prev?.startsWith('blob:')) revoke(prev);
    },
    () => {
      this.viewKey = '';
    }
  );

  private viewBlocked(): boolean {
    return (
      !this.initialised ||
      !this.assetId ||
      this.splitMode ||
      this.showingOriginal ||
      !!this.geometrySession ||
      !!this.maskPreviewLayerId ||
      !!this.colorPicker
    );
  }

  private baseEdge(): number {
    const snap = this.viewSnap;
    if (!snap) return LIVE_EDGE;
    const long = Math.round(Math.max(snap.frame.width, snap.frame.height) * snap.dpr);
    return Math.max(LIVE_EDGE, Math.min(MAX_EDGE, long));
  }

  clearView(): void {
    this.viewFlight.cancel();
    if (this.idleTimer) {
      clearTimeout(this.idleTimer);
      this.idleTimer = null;
    }
    if (this.viewUrl?.startsWith('blob:')) revoke(this.viewUrl);
    this.viewUrl = null;
    this.viewRoi = null;
    this.viewNat = null;
    this.viewFullEdge = 0;
    this.viewKey = '';
  }

  private submitView(): void {
    const snap = this.viewSnap;
    if (!snap) return;
    const visible = visibleRegion(snap.frame, snap.viewW, snap.viewH);
    if (!visible) return;
    const req = renderRequest({
      frame: snap.frame,
      visible,
      dpr: snap.dpr,
      srcLong: this.srcLong,
      serverMaxEdge: VIEW_MAX_EDGE,
      haveRoi: this.viewRoi,
      haveFullEdge: this.viewFullEdge
    });
    if (!req) return;
    const key = `${req.roi.join(',')}:${req.maxEdge}`;
    if (key === this.viewKey) return;
    this.viewKey = key;
    this.viewFlight.submit(req);
  }

  private scheduleIdleRender(): void {
    if (this.idleTimer) clearTimeout(this.idleTimer);
    this.idleTimer = setTimeout(() => {
      this.idleTimer = null;
      if (!this.initialised) return;
      if (this.splitMode) {
        this.flight.submit({
          edits: $state.snapshot(this.edits),
          maxEdge: this.baseEdge(),
          previewMode: 'none'
        });
        return;
      }
      if (this.viewBlocked()) return;
      this.submitView();
    }, VIEW_DEBOUNCE_MS);
  }

  onViewChange = (snap: ViewSnapshot): void => {
    this.viewSnap = snap;
    if (this.viewBlocked()) return;
    this.scheduleIdleRender();
  };

  toggleSplit = (): void => {
    if (this.geometrySession) return;
    this.clearView();
    this.splitMode = !this.splitMode;
    if (this.splitMode) {
      this.flight.submit({
        edits: $state.snapshot(this.edits),
        maxEdge: this.baseEdge(),
        previewMode: 'none'
      });
      this.refreshOriginal();
    } else {
      this.originalFlight.cancel();
      if (this.originalUrl?.startsWith('blob:')) revoke(this.originalUrl);
      this.originalUrl = null;
      this.originalEdge = 0;
      this.originalGeomKey = '';
      this.scheduleIdleRender();
    }
  };

  private proofOptions(): ProofOptions {
    return {
      colorSpace: previewColorSpace(this.proofSpace, this.gamutWarn, displayGamutIsWide()),
      gamutWarn: this.gamutWarn,
      clipWarn: ui.clipWarn
    };
  }

  private reproof(): void {
    if (!this.initialised || !this.assetId) return;
    this.clearView();
    this.flight.submit({
      edits: $state.snapshot(this.edits),
      maxEdge: LIVE_EDGE,
      previewMode: 'none'
    });
    this.scheduleIdleRender();
    if (this.splitMode) this.refreshOriginal(true);
  }

  setProofSpace = (space: ColorSpaceOpt): void => {
    if (this.proofSpace === space) return;
    this.proofSpace = space;
    this.reproof();
  };

  toggleGamutWarn = (): void => {
    this.gamutWarn = !this.gamutWarn;
    this.reproof();
  };

  toggleClipWarn = (): void => {
    ui.toggleClipWarn();
    this.reproof();
  };

  setSplitPos = (p: number): void => {
    this.splitPos = Math.min(1, Math.max(0, p));
  };

  private refreshOriginal(force = false): void {
    if (!this.splitMode || !this.assetId) return;
    const edge = this.baseEdge();
    const snap = $state.snapshot(this.edits);
    const geomKey = JSON.stringify({
      g: snap.geometry,
      l: snap.lens,
      d: snap.color.dcp
    });
    if (
      !force &&
      this.originalEdge === edge &&
      this.originalGeomKey === geomKey &&
      this.originalUrl
    )
      return;
    this.originalFlight.submit({ edge, geomKey });
  }

  async load(id: string): Promise<void> {
    if (this.assetId === id && this.initialised) return;
    await this.finishGeometrySession();
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
      this.flight.submit({
        edits: $state.snapshot(this.edits),
        maxEdge: LIVE_EDGE,
        previewMode: 'none'
      });
      this.scheduleIdleRender();
    } catch (e) {
      this.error = (e as Error).message;
    }
  }

  unload(): void {
    this.flight.cancel();
    this.originalFlight.cancel();
    this.clearView();
    this.viewSnap = null;
    this.srcLong = Number.POSITIVE_INFINITY;
    if (this.geometrySession) {
      if (this.geometrySession.pinnedUrl) revoke(this.geometrySession.pinnedUrl);
      this.geometrySession = null;
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
    this.activeLayerId = null;
    this.activeMaskComponentId = null;
    this.maskPreviewLayerId = null;
    this.activeRetouchId = null;
    this.retouchAnchor = null;
    this.colorPicker = null;
    this.brushBuffers = {};
    this.brushBufferSource = {};
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
    this.previewUrl = persistedPreviewUrl(this.assetId, MAX_EDGE, ui.clipWarn) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
  }

  showOriginal(): void {
    if (!this.initialised) return;
    this.clearView();
    const snap = $state.snapshot(this.edits) as Edits;
    const edits = originalPreviewEdits(snap);
    this.flight.submit({ edits, maxEdge: this.baseEdge(), previewMode: 'none' });
  }

  onLive = (): void => {
    if (!this.initialised) return;
    this.clearView();
    this.flight.submit({
      edits: $state.snapshot(this.edits),
      maxEdge: LIVE_EDGE,
      previewMode: 'none'
    });
    this.scheduleIdleRender();
  };

  onPreview = (mode: PreviewMode): void => {
    if (!this.initialised) return;
    this.clearView();
    this.flight.submit({
      edits: $state.snapshot(this.edits),
      maxEdge: LIVE_EDGE,
      previewMode: mode
    });
  };

  endPreview = (): void => {
    if (!this.initialised) return;
    this.onLive();
  };

  onCommit = async (action?: string): Promise<void> => {
    if (!this.initialised || !this.assetId) return;
    if (!this.skipHistory) this.pushHistory();
    if (this.maskPreviewLayerId) {
      this.onPreview(maskWeightPreview(this.maskPreviewLayerId));
    } else {
      this.onLive();
    }
    const effectiveAction = this.skipHistory ? undefined : action;
    this.lastSaveAction = effectiveAction;
    this.saving = true;
    try {
      if (isIdentity(this.edits)) {
        await deleteEdits(this.assetId, effectiveAction);
        this.savedHash = '';
      } else {
        const saved = await putEdits(
          this.assetId,
          $state.snapshot(this.edits),
          this.savedHash,
          effectiveAction
        );
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

  retrySave = (): Promise<void> => this.onCommit(this.lastSaveAction);

  onReset = async (): Promise<void> => {
    if (!this.assetId) return;
    this.edits = resetDevelopEdits(this.edits);
    await this.onCommit('Reset');
  };

  copyEdits = (): void => {
    if (isIdentity(this.edits)) return;
    copyDialog.show($state.snapshot(this.edits) as Edits);
  };

  pasteEdits = async (): Promise<void> => {
    const snap = clipboard.snapshot();
    if (!snap || !this.initialised) return;
    this.edits = applyCopySections(this.edits, snap.edits, snap.sections);
    this.onLive();
    await this.onCommit('Paste');
  };

  hasClipboard = $derived(clipboard.has);

  applyPreset = async (
    manifest: EditManifest,
    opts: { includeGeometry: boolean; includeMasks: boolean },
    name?: string
  ): Promise<void> => {
    if (!this.initialised) return;
    const incoming = manifestToEdits(manifest);
    this.edits = {
      basic: incoming.basic,
      tone: incoming.tone,
      color: incoming.color,
      detail: incoming.detail,
      effects: incoming.effects,
      lens: incoming.lens,
      geometry: opts.includeGeometry ? incoming.geometry : this.edits.geometry,
      masks: opts.includeMasks ? incoming.masks : this.edits.masks,
      retouch: this.edits.retouch
    };
    this.onLive();
    await this.onCommit(name ? `Preset: ${name}` : 'Preset');
  };

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
      await this.onCommit('Auto');
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
    if (this.colorPicker && this.colorPicker.layerId !== id) this.cancelColorPicker();
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
    if (this.colorPicker && this.colorPicker.componentId !== id) this.cancelColorPicker();
    this.activeMaskComponentId = id;
  };

  setMaskComponentFeather = (layerId: string, componentId: string, feather: number): void => {
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

  retouchFull = $derived(this.edits.retouch.length >= MAX_RETOUCH_STROKES);

  addRetouchStroke = async (stroke: RetouchStroke): Promise<void> => {
    if (!this.initialised || this.edits.retouch.length >= MAX_RETOUCH_STROKES) return;
    this.edits.retouch.push(stroke);
    this.activeRetouchId = stroke.id;
    this.onLive();
    await this.onCommit('Retouch');
  };

  updateRetouchStroke = (id: string, patch: Partial<RetouchStroke>): void => {
    const i = this.edits.retouch.findIndex((s) => s.id === id);
    if (i < 0) return;
    this.edits.retouch[i] = { ...this.edits.retouch[i], ...patch };
  };

  setRetouchStroke = async (
    id: string,
    patch: Partial<RetouchStroke>,
    commit: boolean
  ): Promise<void> => {
    this.updateRetouchStroke(id, patch);
    this.onLive();
    if (commit) await this.onCommit('Retouch');
  };

  commitRetouch = async (): Promise<void> => {
    await this.onCommit('Retouch');
  };

  removeRetouchStroke = async (id: string): Promise<void> => {
    const i = this.edits.retouch.findIndex((s) => s.id === id);
    if (i < 0) return;
    this.edits.retouch.splice(i, 1);
    if (this.activeRetouchId === id) this.activeRetouchId = null;
    this.onLive();
    await this.onCommit('Remove Retouch');
  };

  toggleRetouchStroke = async (id: string): Promise<void> => {
    const s = this.edits.retouch.find((r) => r.id === id);
    if (!s) return;
    s.enabled = !s.enabled;
    this.onLive();
    await this.onCommit(s.enabled ? 'Enable Retouch' : 'Disable Retouch');
  };

  clearRetouch = async (): Promise<void> => {
    if (this.edits.retouch.length === 0) return;
    this.edits.retouch = [];
    this.activeRetouchId = null;
    this.onLive();
    await this.onCommit('Clear Retouch');
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

  beginColorPicker = (layerId: string, componentId: string): void => {
    const layer = this.edits.masks.find((item) => item.id === layerId);
    const component = layer?.components.find((item) => item.id === componentId);
    if (!component || component.kind.kind !== 'color_range') return;
    this.clearView();
    if (this.splitMode) this.toggleSplit();
    this.maskPreviewLayerId = null;
    this.colorPicker = { layerId, componentId, ready: false };
    const edits = $state.snapshot(this.edits) as Edits;
    this.flight.submit({
      edits: { ...edits, masks: [] },
      maxEdge: this.baseEdge(),
      previewMode: 'none',
      purpose: 'color-picker'
    });
  };

  cancelColorPicker = (): void => {
    if (!this.colorPicker) return;
    this.colorPicker = null;
    this.onLive();
  };

  commitColorSample = async (sampleRgb: [number, number, number]): Promise<void> => {
    const picker = this.colorPicker;
    if (!picker) return;
    const layer = this.edits.masks.find((item) => item.id === picker.layerId);
    const component = layer?.components.find((item) => item.id === picker.componentId);
    if (!component || component.kind.kind !== 'color_range') {
      this.cancelColorPicker();
      return;
    }
    this.colorPicker = null;
    this.updateMaskComponentKind(
      picker.layerId,
      picker.componentId,
      { ...component.kind, sample_rgb: sampleRgb },
      false
    );
    await this.commitMasks();
  };

  addMaskLayer = async (kind: MaskComponentKind = defaultLinear()): Promise<string | null> => {
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    const layer = makeLayer(nextLayerName(this.edits.masks), this.edits.masks.length, kind);
    this.edits = { ...this.edits, masks: [...this.edits.masks, layer] };
    this.activeLayerId = layer.id;
    this.activeMaskComponentId = layer.components[0]?.id ?? null;
    await this.onCommit('Masks');
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
    await this.onCommit('Masks');
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
    await this.onCommit('Masks');
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
    await this.onCommit('Masks');
  };

  reorderMaskComponent = async (layerId: string, id: string, toIndex: number): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return;
    const from = layer.components.findIndex((c) => c.id === id);
    if (from < 0) return;
    const components = [...layer.components];
    const [comp] = components.splice(from, 1);
    const clamped = Math.max(0, Math.min(toIndex, components.length));
    components.splice(clamped, 0, clamped === 0 ? { ...comp, mode: 'add' } : comp);
    this.patchMaskLayer(layerId, { components }, false);
    await this.onCommit('Masks');
  };

  patchMaskLayer = (id: string, patch: Partial<MaskLayer>, live = true): void => {
    const masks = this.edits.masks.map((l) => (l.id === id ? { ...l, ...patch } : l));
    this.edits = { ...this.edits, masks };
    if (!live) return;
    if (this.maskPreviewLayerId === id) {
      this.onPreview(maskWeightPreview(id));
    } else {
      this.onLive();
    }
  };

  toggleMaskLayerEnabled = async (id: string): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { enabled: !layer.enabled }, false);
    await this.onCommit('Masks');
  };

  renameMaskLayer = async (id: string, name: string): Promise<void> => {
    this.patchMaskLayer(id, { name }, false);
    await this.onCommit('Masks');
  };

  setMaskLayerColor = async (id: string, color: string): Promise<void> => {
    this.patchMaskLayer(id, { color }, false);
    await this.onCommit('Masks');
  };

  setMaskLayerAmount = (id: string, amount: number): void => {
    this.patchMaskLayer(id, { amount: Math.max(0, Math.min(1, amount)) }, true);
  };

  toggleMaskLayerInvert = async (id: string): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { invert: !layer.invert }, false);
    await this.onCommit('Masks');
  };

  setMaskLayerEdit = (id: string, key: MaskedEditKey, value: number): void => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { edits: setMaskedEdit(layer.edits, key, value) }, true);
  };

  resetMaskLayerEdits = async (id: string): Promise<void> => {
    this.patchMaskLayer(id, { amount: 1, edits: {} }, false);
    await this.onCommit('Masks');
  };

  beginPolygon = (layerId: string | null, mode: MaskComponentMode = 'add'): void => {
    this.setActiveMaskComponent(null);
    this.clickTool = { active: false, negative: false, box: false, layerId: null, mode: 'add' };
    this.polygonDraft = { layerId, mode, points: [] };
  };

  addPolygonPoint = (point: Vec2f): void => {
    const draft = this.polygonDraft;
    if (!draft || draft.points.length >= MAX_POLYGON_POINTS) return;
    this.polygonDraft = { ...draft, points: [...draft.points, point] };
  };

  undoPolygonPoint = (): void => {
    const draft = this.polygonDraft;
    if (!draft || draft.points.length === 0) return;
    this.polygonDraft = { ...draft, points: draft.points.slice(0, -1) };
  };

  cancelPolygon = (): void => {
    this.polygonDraft = null;
  };

  finishPolygon = async (): Promise<void> => {
    const draft = this.polygonDraft;
    if (!draft || draft.points.length < 3) return;
    this.polygonDraft = null;
    const kind: MaskComponentKind = {
      kind: 'polygon',
      points: draft.points,
      feather: 0.05
    };
    if (draft.layerId && this.edits.masks.some((l) => l.id === draft.layerId)) {
      await this.addMaskComponent(draft.layerId, kind, draft.mode);
      return;
    }
    await this.addMaskLayer(kind);
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
    await this.onCommit('Masks');
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
    delete this.brushBufferSource[componentId];
    await this.onCommit('Masks');
  };

  patchMaskComponent = (
    layerId: string,
    componentId: string,
    patch: Partial<MaskComponent>,
    live = true
  ): void => {
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return;
    const components = layer.components.map((c) => (c.id === componentId ? { ...c, ...patch } : c));
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
    await this.onCommit('Masks');
  };

  setBrushTool = (
    patch: Partial<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>
  ): void => {
    this.brushTool = { ...this.brushTool, ...patch };
  };

  setRetouchTool = (
    patch: Partial<{ mode: RetouchMode; size: number; hardness: number; opacity: number }>
  ): void => {
    this.retouchTool = { ...this.retouchTool, ...patch };
  };

  setRetouchMode = (mode: RetouchMode): void => {
    this.retouchTool = { ...this.retouchTool, mode };
    if (this.activeRetouchId) void this.setRetouchStroke(this.activeRetouchId, { mode }, true);
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
    const key = rasterId ?? '';
    const existing = this.brushBuffers[componentId];
    if (existing && this.brushBufferSource[componentId] === key) return existing;
    if (rasterId) {
      try {
        const r = await fetchRaster(rasterId);
        const buf: BrushBuffer = { width: r.width, height: r.height, bytes: r.bytes };
        this.brushBuffers = { ...this.brushBuffers, [componentId]: buf };
        this.brushBufferSource[componentId] = key;
        return buf;
      } catch {
        // fall through to blank
      }
    }
    const { width, height } = this.brushDims();
    const buf = blankBuffer(width, height);
    this.brushBuffers = { ...this.brushBuffers, [componentId]: buf };
    this.brushBufferSource[componentId] = key;
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
    await this.onCommit('Masks');
    return layer.id;
  };

  private failMask = (e: unknown, retry: () => Promise<unknown>): void => {
    this.maskError = (e as Error).message;
    this.maskRetry = retry;
  };

  retryMask = async (): Promise<void> => {
    const retry = this.maskRetry;
    if (!retry) return;
    this.maskError = null;
    this.maskRetry = null;
    await retry();
  };

  dismissMaskError = (): void => {
    this.maskError = null;
    this.maskRetry = null;
  };

  addGeneratedComponent = async (
    layerId: string,
    kind: MaskKind,
    mode: MaskComponentMode = 'add',
    maskClass?: string,
    invert = false
  ): Promise<string | null> => {
    const assetId = this.assetId;
    if (!assetId || this.maskGenerating) return null;
    const cap = maskCapacity(this.edits, layerId);
    if (cap.componentsFull || cap.totalFull) return null;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return null;
    this.maskGenerating = true;
    this.maskError = null;
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
      this.patchMaskLayer(layerId, { components: [...layer.components, comp] }, false);
      this.activeMaskComponentId = comp.id;
      await this.ensureBrushBuffer(comp.id, res.raster_id);
      await this.onCommit('Masks');
      return comp.id;
    } catch (e) {
      this.failMask(e, () => this.addGeneratedComponent(layerId, kind, mode, maskClass, invert));
      return null;
    } finally {
      this.maskGenerating = false;
    }
  };

  addGeneratedLayer = async (
    kind: MaskKind,
    maskClass?: string,
    invert = false
  ): Promise<string | null> => {
    const assetId = this.assetId;
    if (!assetId || this.maskGenerating) return null;
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    this.maskGenerating = true;
    this.maskError = null;
    try {
      const res = await generateMask(assetId, kind, 0, 0, maskClass);
      const layer = makeGeneratedLayer(
        nextLayerName(this.edits.masks),
        this.edits.masks.length,
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
      this.edits = { ...this.edits, masks: [...this.edits.masks, layer] };
      this.activeLayerId = layer.id;
      const compId = layer.components[0]?.id ?? null;
      this.activeMaskComponentId = compId;
      if (compId) await this.ensureBrushBuffer(compId, res.raster_id);
      await this.onCommit('Masks');
      return layer.id;
    } catch (e) {
      this.failMask(e, () => this.addGeneratedLayer(kind, maskClass, invert));
      return null;
    } finally {
      this.maskGenerating = false;
    }
  };

  rebakeGeneratedComponent = async (
    layerId: string,
    componentId: string,
    grow: number,
    feather: number,
    range?: MaskRange
  ): Promise<void> => {
    const assetId = this.assetId;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    const comp = layer?.components.find((c) => c.id === componentId);
    if (!assetId || !comp?.generated || this.maskGenerating) return;
    this.maskGenerating = true;
    this.maskError = null;
    try {
      const res = await rebakeMask(assetId, comp.generated.prob_raster_id, grow, feather, range);
      await this.ensureBrushBuffer(componentId, res.raster_id);
      this.patchMaskComponent(
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
      await this.onCommit('Masks');
    } catch (e) {
      this.failMask(e, () =>
        this.rebakeGeneratedComponent(layerId, componentId, grow, feather, range)
      );
    } finally {
      this.maskGenerating = false;
    }
  };

  clickRefineComponent = async (
    layerId: string,
    componentId: string,
    points: ClickPoint[]
  ): Promise<void> => {
    const assetId = this.assetId;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    const comp = layer?.components.find((c) => c.id === componentId);
    if (!assetId || !comp?.generated || this.maskGenerating) return;
    if (points.length === 0) return;
    const grow = comp.generated.grow;
    const feather = comp.generated.feather;
    this.maskGenerating = true;
    this.maskError = null;
    try {
      const res = await clickMask(assetId, points, grow, feather);
      await this.ensureBrushBuffer(componentId, res.raster_id);
      this.patchMaskComponent(
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
      await this.onCommit('Masks');
    } catch (e) {
      this.failMask(e, () => this.clickRefineComponent(layerId, componentId, points));
    } finally {
      this.maskGenerating = false;
    }
  };

  addClickLayer = async (points: ClickPoint[], bbox?: MaskBox): Promise<string | null> => {
    const assetId = this.assetId;
    if (!assetId || this.maskGenerating || (points.length === 0 && !bbox)) return null;
    const cap = maskCapacity(this.edits, null);
    if (cap.layersFull || cap.totalFull) return null;
    this.maskGenerating = true;
    this.maskError = null;
    try {
      const res = await clickMask(assetId, points, 0, 0, undefined, false, bbox);
      const layer = makeGeneratedLayer(
        nextLayerName(this.edits.masks),
        this.edits.masks.length,
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
      this.edits = { ...this.edits, masks: [...this.edits.masks, layer] };
      this.activeLayerId = layer.id;
      const compId = layer.components[0]?.id ?? null;
      this.activeMaskComponentId = compId;
      if (compId) await this.ensureBrushBuffer(compId, res.raster_id);
      await this.onCommit('Masks');
      return layer.id;
    } catch (e) {
      this.failMask(e, () => this.addClickLayer(points, bbox));
      return null;
    } finally {
      this.maskGenerating = false;
    }
  };

  removeClickPoint = async (index: number): Promise<void> => {
    if (this.maskGenerating) return;
    const layer = this.activeLayerId
      ? (this.edits.masks.find((l) => l.id === this.activeLayerId) ?? null)
      : null;
    const comp = layer?.components.find((c) => c.id === this.activeMaskComponentId) ?? null;
    if (!layer || !comp || comp.generated?.kind !== 'click') return;
    const points = comp.generated.points ?? [];
    if (index < 0 || index >= points.length) return;
    const next = points.filter((_, i) => i !== index);
    if (next.length === 0) {
      await this.removeMaskComponent(layer.id, comp.id);
      return;
    }
    await this.clickRefineComponent(layer.id, comp.id, next);
  };

  addClickComponent = async (
    layerId: string,
    points: ClickPoint[],
    mode: MaskComponentMode = 'add',
    bbox?: MaskBox
  ): Promise<string | null> => {
    const assetId = this.assetId;
    if (!assetId || this.maskGenerating || (points.length === 0 && !bbox)) return null;
    const cap = maskCapacity(this.edits, layerId);
    if (cap.componentsFull || cap.totalFull) return null;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    if (!layer) return null;
    this.maskGenerating = true;
    this.maskError = null;
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
      this.patchMaskLayer(layerId, { components: [...layer.components, comp] }, false);
      this.activeMaskComponentId = comp.id;
      await this.ensureBrushBuffer(comp.id, res.raster_id);
      await this.onCommit('Masks');
      return comp.id;
    } catch (e) {
      this.failMask(e, () => this.addClickComponent(layerId, points, mode, bbox));
      return null;
    } finally {
      this.maskGenerating = false;
    }
  };

  clickRefineRaster = async (
    layerId: string,
    componentId: string,
    points: ClickPoint[],
    subtract: boolean,
    bbox?: MaskBox
  ): Promise<void> => {
    const assetId = this.assetId;
    const layer = this.edits.masks.find((l) => l.id === layerId);
    const comp = layer?.components.find((c) => c.id === componentId);
    if (!assetId || !comp || comp.kind.kind !== 'brush' || this.maskGenerating) return;
    const base = comp.generated?.prob_raster_id ?? comp.kind.raster_id;
    const grow = comp.generated?.grow ?? 0;
    const feather = comp.generated?.feather ?? 0;
    this.maskGenerating = true;
    this.maskError = null;
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
      await this.ensureBrushBuffer(componentId, res.raster_id);
      this.patchMaskComponent(
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
      await this.onCommit('Masks');
    } catch (e) {
      this.failMask(e, () => this.clickRefineRaster(layerId, componentId, points, subtract, bbox));
    } finally {
      this.maskGenerating = false;
    }
  };

  addClickPoint = async (x: number, y: number, positive: boolean): Promise<void> => {
    if (this.maskGenerating) return;
    const layer = this.activeLayerId
      ? (this.edits.masks.find((l) => l.id === this.activeLayerId) ?? null)
      : null;
    const comp = layer?.components.find((c) => c.id === this.activeMaskComponentId) ?? null;
    const point = { x, y, positive };
    if (layer && comp?.generated?.kind === 'click' && (comp.generated.points ?? []).length > 0) {
      const points = [...(comp.generated.points ?? []), point];
      await this.clickRefineComponent(layer.id, comp.id, points);
      return;
    }
    if (layer && comp && comp.kind.kind === 'brush') {
      await this.clickRefineRaster(layer.id, comp.id, [point], !positive);
      return;
    }
    if (!positive) return;
    const target = this.clickTool.layerId;
    if (target && this.edits.masks.some((l) => l.id === target)) {
      await this.addClickComponent(target, [point], this.clickTool.mode);
      return;
    }
    await this.addClickLayer([point]);
  };

  addClickBox = async (bbox: MaskBox): Promise<void> => {
    if (this.maskGenerating) return;
    const layer = this.activeLayerId
      ? (this.edits.masks.find((l) => l.id === this.activeLayerId) ?? null)
      : null;
    const comp = layer?.components.find((c) => c.id === this.activeMaskComponentId) ?? null;
    if (layer && comp && comp.kind.kind === 'brush') {
      await this.clickRefineRaster(layer.id, comp.id, [], this.clickTool.negative, bbox);
      return;
    }
    const target = this.clickTool.layerId;
    if (target && this.edits.masks.some((l) => l.id === target)) {
      await this.addClickComponent(target, [], this.clickTool.mode, bbox);
      return;
    }
    await this.addClickLayer([], bbox);
  };

  addBrushComponent = async (
    layerId: string,
    mode: MaskComponentMode = 'add'
  ): Promise<string | null> => {
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
    const id = await this.addMaskComponent(layerId, defaultBrush(rasterId), mode);
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
      this.brushBufferSource[componentId] = meta.raster_id;
      this.updateMaskComponentKind(
        layerId,
        componentId,
        { kind: 'brush', raster_id: meta.raster_id },
        false
      );
      if (comp.generated && !comp.generated.painted) {
        this.patchMaskComponent(
          layerId,
          componentId,
          { generated: { ...comp.generated, painted: true } },
          false
        );
      }
      await this.onCommit('Masks');
    } catch (e) {
      this.error = (e as Error).message;
    }
  };

  private syncBrowsing(): void {
    if (!this.assetId || !this.asset) return;
    browsing.patch(this.assetId, {
      isFavorite: this.asset.isFavorite,
      exifInfo: this.asset.exifInfo ?? null,
      tags: this.asset.tags
    });
  }

  toggleFavorite = async (): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (!(await metadataConsent.gate())) return;
    const prev = this.asset.isFavorite;
    this.asset = { ...this.asset, isFavorite: !prev };
    try {
      const updated = await updateAsset(this.assetId, { isFavorite: !prev });
      this.asset = { ...updated, tags: this.asset.tags };
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, isFavorite: prev };
      this.error = (e as Error).message;
    }
  };

  setRating = async (rating: number | null): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (!(await metadataConsent.gate())) return;
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
      this.asset = { ...updated, tags: this.asset.tags };
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, exifInfo: prevExif };
      this.error = (e as Error).message;
    }
  };

  addTag = async (tag: TagRef): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (this.asset.tags.some((t) => t.id === tag.id)) return;
    if (!(await metadataConsent.gate())) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: [...prev, tag] };
    try {
      await addTagToAsset(tag.id, this.assetId);
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  removeTag = async (tagId: string): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (!(await metadataConsent.gate())) return;
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: prev.filter((t) => t.id !== tagId) };
    try {
      await removeTagFromAsset(tagId, this.assetId);
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      this.error = (e as Error).message;
    }
  };

  toggleReject = async (): Promise<void> => {
    if (!this.assetId || !this.asset) return;
    if (!(await metadataConsent.gate())) return;
    const rejectTag = await ensureRejectTag();
    if (!rejectTag) {
      this.error = 'reject: could not create tag';
      return;
    }
    const next = !isRejected(this.asset);
    const prev = this.asset.tags;
    this.asset = { ...this.asset, tags: setRejectedTags(prev, rejectTag, next) };
    if (next) rejected.add(this.assetId, rejectTag);
    else rejected.remove(this.assetId);
    try {
      if (next) await addTagToAsset(rejectTag.id, this.assetId);
      else await removeTagFromAsset(rejectTag.id, this.assetId);
      this.syncBrowsing();
    } catch (e) {
      if (this.asset) this.asset = { ...this.asset, tags: prev };
      if (next) rejected.remove(this.assetId);
      else rejected.add(this.assetId, rejectTag);
      this.error = (e as Error).message;
    }
  };

  clearFlags = async (): Promise<void> => {
    if (!this.asset) return;
    if (this.asset.isFavorite) await this.toggleFavorite();
    if (this.asset && isRejected(this.asset)) await this.toggleReject();
  };

  createAndAddTag = async (value: string): Promise<TagRef | null> => {
    if (!this.assetId || !this.asset) return null;
    if (!(await metadataConsent.gate())) return null;
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
          this.syncBrowsing();
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

  private async loadMeta(metaId: string): Promise<void> {
    if (!this.assetId) return;
    try {
      this.meta = await getPreviewMeta(this.assetId, metaId);
      const long = Math.max(this.meta.source_w, this.meta.source_h);
      if (Number.isFinite(long) && long > 0) this.srcLong = long;
    } catch {
      this.meta = null;
    }
  }

  startGeometrySession = (): void => {
    if (!this.assetId || !this.initialised || this.geometrySession) return;
    this.clearView();
    const baseEdits = $state.snapshot(this.edits) as Edits;
    const sessionId = ++this.geometrySessionId;
    this.geometrySession = {
      id: sessionId,
      pinnedUrl: null,
      pinnedReady: false,
      srcW: 0,
      srcH: 0,
      draftRotate: baseEdits.geometry.rotate,
      draftFlipH: baseEdits.geometry.flip_h,
      draftFlipV: baseEdits.geometry.flip_v,
      draftAngle: baseEdits.geometry.rotate_angle,
      draftCrop: baseEdits.geometry.crop ?? FULL_CROP,
      draftAspect: baseEdits.geometry.aspect,
      draftPerspective: baseEdits.geometry.perspective ?? neutralPerspective(),
      userEditedCrop: baseEdits.geometry.crop !== null
    };
    void this.loadGeometryPreview(baseEdits, sessionId);
  };

  private async loadGeometryPreview(baseEdits: Edits, sessionId: number): Promise<void> {
    if (!this.assetId) return;
    const canonical: Edits = {
      ...baseEdits,
      geometry: {
        ...baseEdits.geometry,
        rotate: 0,
        flip_h: false,
        flip_v: false,
        rotate_angle: 0,
        crop: null,
        perspective: null
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
      const sess = this.geometrySession;
      if (sess?.id !== sessionId || dims.w <= 0 || dims.h <= 0) {
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
      if (this.geometrySession?.id !== sessionId) return;
      this.error = (e as Error).message;
    }
  }

  finishGeometrySession = async (): Promise<void> => {
    const sess = this.geometrySession;
    if (!sess) return;
    if (sess.pinnedUrl) revoke(sess.pinnedUrl);
    this.geometrySession = null;
    const dc = sess.draftCrop;
    const full = dc.x === 0 && dc.y === 0 && dc.w === 1 && dc.h === 1;
    const geometry = {
      ...this.edits.geometry,
      rotate: sess.draftRotate,
      flip_h: sess.draftFlipH,
      flip_v: sess.draftFlipV,
      rotate_angle: sess.draftAngle,
      crop: full ? null : sess.draftCrop,
      aspect: sess.draftAspect,
      perspective: perspectiveIsIdentity(sess.draftPerspective)
        ? null
        : clampPerspective(sess.draftPerspective)
    };
    if (JSON.stringify(this.edits.geometry) === JSON.stringify(geometry)) return;
    this.edits = {
      ...this.edits,
      geometry
    };
    await this.onCommit('Geometry');
  };

  rotateStep = (delta: 90 | 270): void => {
    const sess = this.geometrySession;
    if (sess) {
      sess.draftRotate = ((sess.draftRotate + delta) % 360) as 0 | 90 | 180 | 270;
      const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
      const sw = swapped ? sess.srcH : sess.srcW;
      const sh = swapped ? sess.srcW : sess.srcH;
      const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
      sess.draftCrop =
        ratio !== null
          ? largestInscribedRect(sw, sh, sess.draftAngle, ratio, draftPerspInv(sess))
          : FULL_CROP;
      sess.userEditedCrop = false;
      return;
    }
    this.edits.geometry.rotate = ((this.edits.geometry.rotate + delta) % 360) as 0 | 90 | 180 | 270;
    void this.onCommit('Rotate');
  };

  flipStep = (axis: 'h' | 'v'): void => {
    const sess = this.geometrySession;
    if (sess) {
      if (axis === 'h') sess.draftFlipH = !sess.draftFlipH;
      else sess.draftFlipV = !sess.draftFlipV;
      return;
    }
    if (axis === 'h') this.edits.geometry.flip_h = !this.edits.geometry.flip_h;
    else this.edits.geometry.flip_v = !this.edits.geometry.flip_v;
    void this.onCommit('Flip');
  };

  updateGeometryDraftAngle = (angle: number): void => {
    const sess = this.geometrySession;
    if (!sess) return;
    sess.draftAngle = angle;
    this.refitGeometryDraftCrop(sess);
  };

  updateGeometryDraftPerspective = (patch: Partial<PerspectiveEdits>): void => {
    const sess = this.geometrySession;
    if (!sess) return;
    sess.draftPerspective = limitPerspective(
      sess.draftPerspective,
      clampPerspective({ ...sess.draftPerspective, ...patch })
    );
    this.refitGeometryDraftCrop(sess);
  };

  private refitGeometryDraftCrop(sess: GeometrySession): void {
    const { sw, sh } = sourceDims(sess);
    const angle = sess.draftAngle;
    const persp = draftPerspInv(sess);
    const ratio = aspectRatioFor(sess.draftAspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = refitCropAtAspect(sess.draftCrop, sw, sh, angle, ratio, persp);
      return;
    }
    if (!sess.userEditedCrop) {
      sess.draftCrop = largestInscribedRect(sw, sh, angle, sw / sh, persp);
      return;
    }
    sess.draftCrop = constrainCropRect(sess.draftCrop, sess.draftCrop, sw, sh, angle, persp);
  }

  updateGeometryDraftCrop = (crop: CropRect): void => {
    const sess = this.geometrySession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftCrop = constrainCropRect(
      crop,
      sess.draftCrop,
      sw,
      sh,
      sess.draftAngle,
      draftPerspInv(sess)
    );
    sess.userEditedCrop = true;
  };

  updateGeometryDraftAspect = (aspect: AspectLock): void => {
    const sess = this.geometrySession;
    if (!sess) return;
    const { sw, sh } = sourceDims(sess);
    sess.draftAspect = aspect;
    const ratio = aspectRatioFor(aspect, sw, sh);
    if (ratio !== null) {
      sess.draftCrop = largestInscribedRect(sw, sh, sess.draftAngle, ratio, draftPerspInv(sess));
      sess.userEditedCrop = false;
    }
  };

  resetGeometryDraft = (): void => {
    const sess = this.geometrySession;
    if (!sess) return;
    sess.draftAngle = 0;
    sess.draftAspect = { kind: 'original' };
    sess.draftCrop = FULL_CROP;
    sess.draftPerspective = neutralPerspective();
    sess.userEditedCrop = false;
  };
}

function draftPerspInv(sess: GeometrySession): Mat3 {
  return perspectiveInverse(sess.draftPerspective);
}

function sourceDims(sess: GeometrySession): { sw: number; sh: number } {
  const swapped = sess.draftRotate === 90 || sess.draftRotate === 270;
  return { sw: swapped ? sess.srcH : sess.srcW, sh: swapped ? sess.srcW : sess.srcH };
}

interface GeometrySession {
  id: number;
  pinnedUrl: string | null;
  pinnedReady: boolean;
  srcW: number;
  srcH: number;
  draftRotate: 0 | 90 | 180 | 270;
  draftFlipH: boolean;
  draftFlipV: boolean;
  draftAngle: number;
  draftCrop: CropRect;
  draftAspect: AspectLock;
  draftPerspective: PerspectiveEdits;
  userEditedCrop: boolean;
}

export const editor = new EditorStore();
