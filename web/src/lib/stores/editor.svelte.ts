import {
  neutralEdits,
  originalPreviewEdits,
  resetDevelopEdits,
  isIdentity,
  manifestToEdits,
  effectiveLens,
  MAX_RETOUCH_STROKES,
  type AspectLock,
  type CropRect,
  type Edits,
  type EditManifest,
  type LensEdits,
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
  defaultLinear,
  defaultMaskColor,
  makeComponent,
  MAX_POLYGON_POINTS,
  makeLayer,
  maskCapacity,
  nextLayerName,
  setMaskedEdit
} from '$lib/types/masks';
import type { BrushBuffer } from '$lib/utils/brush';
import type { ClickPoint, MaskBox, MaskKind, MaskRange } from '$lib/api/masks';
import * as maskGen from '$lib/stores/editor/maskGen';
import * as metadata from '$lib/stores/editor/metadata';
import * as geometry from '$lib/stores/editor/geometry.svelte';
import type { GeometrySession } from '$lib/stores/editor/geometry.svelte';
import type { PreviewMeta } from '$lib/types/preview';
import type { AssetDetail, TagRef } from '$lib/types/asset';
import { getEdits, putEdits, deleteEdits, autoEdits, restoreEdits } from '$lib/api/edits';
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
import { getAsset } from '$lib/api/assets';
import { getLensProfile, type LensProfileMatch } from '$lib/api/lensProfile';
import { isRejected } from '$lib/reject';
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
import { errorMessage } from '$lib/utils/errors';
import type { PerspectiveEdits } from '$lib/utils/perspective';

const LIVE_EDGE = 1600;
const MAX_EDGE = 4096;
const VIEW_MAX_EDGE = 65535;
const VIEW_DEBOUNCE_MS = 150;
const MAX_HISTORY = 50;

type ViewSnapshot = { frame: Rect; viewW: number; viewH: number; dpr: number };

type SaveSession = {
  assetId: string;
  hash: string;
  pending: number;
  blocked: boolean;
};

class EditorStore {
  assetId = $state<string | null>(null);
  asset = $state<AssetDetail | null>(null);
  edits = $state<Edits>(neutralEdits());
  previewUrl = $state<string | null>(null);
  meta = $state<PreviewMeta | null>(null);
  lensProfile = $state<LensProfileMatch | null>(null);
  lensProfileError = $state<string | null>(null);
  pending = $state(false);
  saving = $state(false);
  savedHash = $state<string>('');
  saveError = $state<string | null>(null);
  private lastSaveAction: string | undefined = undefined;
  private saveSession: SaveSession | null = null;
  private saveTail: Promise<void> = Promise.resolve();
  exporting = $state(false);
  exportingToImmich = $state(false);
  lastUpload = $state<{ kind: 'success' | 'duplicate' | 'error'; message: string } | null>(null);
  hasEdits = $derived(!isIdentity(this.edits));
  lensView: LensEdits = $derived(
    effectiveLens(this.edits.lens, this.meta?.is_raw ? (this.lensProfile?.edits ?? null) : null)
  );
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

  activeLayerId = $state<string | null>(null);
  activeMaskComponentId = $state<string | null>(null);
  maskGenerating = $state(false);
  maskError = $state<string | null>(null);
  maskRetry: (() => Promise<unknown>) | null = null;
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
  brushBufferSource: Record<string, string> = {};
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
  private viewSnap = $state<ViewSnapshot | null>(null);
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
      if (delivered < args.maxEdge * 0.98) this.srcLong = this.viewFullEdge;
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

  get sourceLong(): number {
    return this.srcLong;
  }

  get viewScale(): number | null {
    const snap = this.viewSnap;
    if (!snap || !this.viewRoi || !this.viewNat) return null;
    const boxLong =
      Math.max(this.viewRoi[2] * snap.frame.width, this.viewRoi[3] * snap.frame.height) * snap.dpr;
    if (boxLong <= 0) return null;
    return Math.max(this.viewNat.w, this.viewNat.h) / boxLong;
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
      this.saveSession = { assetId: id, hash: s.hash, pending: 0, blocked: false };
      this.initialised = true;
      this.pushHistory();
      this.fetchLensProfile(id);
      this.flight.submit({
        edits: $state.snapshot(this.edits),
        maxEdge: LIVE_EDGE,
        previewMode: 'none'
      });
      this.scheduleIdleRender();
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  private fetchLensProfile(id: string): void {
    this.lensProfile = null;
    this.lensProfileError = null;
    getLensProfile(id)
      .then((p) => {
        if (this.assetId === id) this.lensProfile = p;
      })
      .catch((e: unknown) => {
        if (this.assetId === id) {
          this.lensProfileError = e instanceof Error ? e.message : String(e);
        }
      });
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
    this.lensProfile = null;
    this.lensProfileError = null;
    this.assetId = null;
    this.initialised = false;
    this.edits = neutralEdits();
    this.saving = false;
    this.savedHash = '';
    this.saveError = null;
    this.saveSession = null;
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
    const session = this.saveSession;
    if (!this.initialised || !session) return;
    if (!this.skipHistory) this.pushHistory();
    if (this.maskPreviewLayerId) {
      this.onPreview(maskWeightPreview(this.maskPreviewLayerId));
    } else {
      this.onLive();
    }
    const effectiveAction = this.skipHistory ? undefined : action;
    this.lastSaveAction = effectiveAction;
    await this.queueSave(session, $state.snapshot(this.edits) as Edits, effectiveAction);
  };

  private queueSave(session: SaveSession, edits: Edits, action?: string): Promise<void> {
    return this.queueSessionTask(session, () => this.persistSave(session, edits, action));
  }

  private queueSessionTask(session: SaveSession, task: () => Promise<void>): Promise<void> {
    session.pending++;
    if (this.saveSession === session) this.saving = true;
    const queued = this.saveTail.then(async () => {
      if (session.blocked) return;
      await task();
    });
    this.saveTail = queued.catch(() => undefined);
    return queued.finally(() => {
      session.pending--;
      if (this.saveSession === session) this.saving = session.pending > 0;
    });
  }

  restoreHistoryEntry = (entryId: number): Promise<void> => {
    const session = this.saveSession;
    if (!this.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.saveError = null;
    return this.queueSessionTask(session, async () => {
      if (this.saveSession !== session) return;
      const saved = await restoreEdits(session.assetId, entryId);
      if (this.saveSession !== session) return;
      this.edits = saved ? manifestToEdits(saved.manifest) : neutralEdits();
      session.hash = saved?.hash ?? '';
      this.savedHash = session.hash;
      this.error = null;
      this.onLive();
    });
  };

  private async persistSave(session: SaveSession, edits: Edits, action?: string): Promise<void> {
    try {
      if (isIdentity(edits)) {
        await deleteEdits(session.assetId, action);
        session.hash = '';
      } else {
        const saved = await putEdits(session.assetId, edits, session.hash, action);
        session.hash = saved.hash;
      }
      if (this.saveSession === session) {
        this.savedHash = session.hash;
        this.saveError = null;
      }
    } catch (e) {
      session.blocked = true;
      if (this.saveSession !== session) return;
      if (e instanceof ConflictError) {
        const current = e.current as EditRecord | undefined;
        if (current) session.hash = current.hash;
        this.savedHash = session.hash;
        this.saveError = 'Edits changed elsewhere. Local edits kept. Retry to save them.';
        toasts.push('warn', this.saveError);
      } else {
        this.saveError = errorMessage(e);
        this.error = this.saveError;
      }
    }
  }

  retrySave = (): Promise<void> => {
    const session = this.saveSession;
    if (!this.initialised || !session) return Promise.resolve();
    session.blocked = false;
    this.saveError = null;
    return this.queueSave(session, $state.snapshot(this.edits) as Edits, this.lastSaveAction);
  };

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
      this.error = errorMessage(e);
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
      this.error = errorMessage(e);
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
      const m = errorMessage(e);
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
    await this.onCommit(`Add ${layer.name}`);
    return layer.id;
  };

  removeMaskLayer = async (id: string): Promise<void> => {
    const idx = this.edits.masks.findIndex((l) => l.id === id);
    if (idx < 0) return;
    const name = this.edits.masks[idx].name;
    const masks = this.edits.masks.filter((l) => l.id !== id);
    this.edits = { ...this.edits, masks };
    if (this.activeLayerId === id) {
      this.activeLayerId = masks[idx]?.id ?? masks[masks.length - 1]?.id ?? null;
      this.activeMaskComponentId = null;
    }
    if (this.maskPreviewLayerId === id) this.endMaskPreview();
    await this.onCommit(`Delete ${name}`);
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
    await this.onCommit(`Duplicate ${src.name}`);
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
    await this.onCommit('Reorder Masks');
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
    await this.onCommit('Reorder Mask Shapes');
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
    await this.onCommit(layer.enabled ? `Disable ${layer.name}` : `Enable ${layer.name}`);
  };

  renameMaskLayer = async (id: string, name: string): Promise<void> => {
    this.patchMaskLayer(id, { name }, false);
    await this.onCommit('Rename Mask');
  };

  setMaskLayerColor = async (id: string, color: string): Promise<void> => {
    this.patchMaskLayer(id, { color }, false);
    await this.onCommit('Change Mask Color');
  };

  setMaskLayerAmount = (id: string, amount: number): void => {
    this.patchMaskLayer(id, { amount: Math.max(0, Math.min(1, amount)) }, true);
  };

  toggleMaskLayerInvert = async (id: string): Promise<void> => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { invert: !layer.invert }, false);
    await this.onCommit(`Invert ${layer.name}`);
  };

  setMaskLayerEdit = (id: string, key: MaskedEditKey, value: number): void => {
    const layer = this.edits.masks.find((l) => l.id === id);
    if (!layer) return;
    this.patchMaskLayer(id, { edits: setMaskedEdit(layer.edits, key, value) }, true);
  };

  resetMaskLayerEdits = async (id: string): Promise<void> => {
    this.patchMaskLayer(id, { amount: 1, edits: {} }, false);
    await this.onCommit('Reset Mask Adjustments');
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
    await this.onCommit(`Add ${kind.kind.replaceAll('_', ' ')} Shape`);
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
    await this.onCommit('Delete Mask Shape');
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
    await this.onCommit('Adjust Mask');
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

  ensureBrushBuffer = (componentId: string, rasterId: string | null): Promise<BrushBuffer> =>
    maskGen.ensureBrushBuffer(this, componentId, rasterId);

  addBrushLayer = (): Promise<string | null> => maskGen.addBrushLayer(this);

  addBrushComponent = (layerId: string, mode: MaskComponentMode = 'add'): Promise<string | null> =>
    maskGen.addBrushComponent(this, layerId, mode);

  commitBrushStroke = (layerId: string, componentId: string): Promise<void> =>
    maskGen.commitBrushStroke(this, layerId, componentId);

  addGeneratedComponent = (
    layerId: string,
    kind: MaskKind,
    mode: MaskComponentMode = 'add',
    maskClass?: string,
    invert = false
  ): Promise<string | null> =>
    maskGen.addGeneratedComponent(this, layerId, kind, mode, maskClass, invert);

  addGeneratedLayer = (
    kind: MaskKind,
    maskClass?: string,
    invert = false
  ): Promise<string | null> => maskGen.addGeneratedLayer(this, kind, maskClass, invert);

  rebakeGeneratedComponent = (
    layerId: string,
    componentId: string,
    grow: number,
    feather: number,
    range?: MaskRange
  ): Promise<void> =>
    maskGen.rebakeGeneratedComponent(this, layerId, componentId, grow, feather, range);

  clickRefineComponent = (
    layerId: string,
    componentId: string,
    points: ClickPoint[]
  ): Promise<void> => maskGen.clickRefineComponent(this, layerId, componentId, points);

  clickRefineRaster = (
    layerId: string,
    componentId: string,
    points: ClickPoint[],
    subtract: boolean,
    bbox?: MaskBox
  ): Promise<void> => maskGen.clickRefineRaster(this, layerId, componentId, points, subtract, bbox);

  addClickLayer = (points: ClickPoint[], bbox?: MaskBox): Promise<string | null> =>
    maskGen.addClickLayer(this, points, bbox);

  addClickComponent = (
    layerId: string,
    points: ClickPoint[],
    mode: MaskComponentMode = 'add',
    bbox?: MaskBox
  ): Promise<string | null> => maskGen.addClickComponent(this, layerId, points, mode, bbox);

  addClickPoint = (x: number, y: number, positive: boolean): Promise<void> =>
    maskGen.addClickPoint(this, x, y, positive);

  addClickBox = (bbox: MaskBox): Promise<void> => maskGen.addClickBox(this, bbox);

  removeClickPoint = (index: number): Promise<void> => maskGen.removeClickPoint(this, index);

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

  toggleFavorite = (): Promise<void> => metadata.toggleFavorite(this);

  setRating = (rating: number | null): Promise<void> => metadata.setRating(this, rating);

  addTag = (tag: TagRef): Promise<void> => metadata.addTag(this, tag);

  removeTag = (tagId: string): Promise<void> => metadata.removeTag(this, tagId);

  toggleReject = (): Promise<void> => metadata.toggleReject(this);

  createAndAddTag = (value: string): Promise<TagRef | null> =>
    metadata.createAndAddTag(this, value);

  clearFlags = async (): Promise<void> => {
    if (!this.asset) return;
    if (this.asset.isFavorite) await this.toggleFavorite();
    if (this.asset && isRejected(this.asset)) await this.toggleReject();
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

  startGeometrySession = (): void => geometry.startSession(this);

  finishGeometrySession = (): Promise<void> => geometry.finishSession(this);

  rotateStep = (delta: 90 | 270): void => geometry.rotateStep(this, delta);

  flipStep = (axis: 'h' | 'v'): void => geometry.flipStep(this, axis);

  updateGeometryDraftAngle = (angle: number): void => geometry.updateDraftAngle(this, angle);

  updateGeometryDraftPerspective = (patch: Partial<PerspectiveEdits>): void =>
    geometry.updateDraftPerspective(this, patch);

  updateGeometryDraftCrop = (crop: CropRect): void => geometry.updateDraftCrop(this, crop);

  updateGeometryDraftAspect = (aspect: AspectLock): void =>
    geometry.updateDraftAspect(this, aspect);

  resetGeometryDraft = (): void => geometry.resetDraft(this);
}

export const editor = new EditorStore();
