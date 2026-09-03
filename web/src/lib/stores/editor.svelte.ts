import {
  neutralEdits,
  originalPreviewEdits,
  resetDevelopEdits,
  isIdentity,
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
import { manifestToEdits } from '$lib/edits/manifest';
import { defaultLinear, maskCapacity } from '$lib/types/masks';
import type { BrushBuffer } from '$lib/utils/brush';
import type { ClickPoint, MaskBox, MaskKind, MaskRange } from '$lib/api/masks';
import * as exportActions from '$lib/stores/editor/export.svelte';
import * as maskLayers from '$lib/stores/editor/maskLayers.svelte';
import * as maskGen from '$lib/stores/editor/maskGen';
import * as metadata from '$lib/stores/editor/metadata';
import * as retouch from '$lib/stores/editor/retouch';
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
import { type ColorSpaceOpt, type ExportOptions, type ImmichExportOptions } from '$lib/api/export';
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
import { errorMessage } from '$lib/utils/errors';
import { cachedFaceData, loadFaceData } from '$lib/stores/zoomTargets';
import { viewTransform } from '$lib/utils/canvasCoords';
import {
  faceTargets,
  nextTargetIndex,
  panForTarget,
  sharpestPoint,
  type ZoomTarget
} from '$lib/utils/zoomTarget';
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
  lastImmichOpts: ImmichExportOptions | null = null;
  lastExportOpts: ExportOptions | null = null;
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
  private baseImage: HTMLImageElement | null = null;
  private zoomTargetIndex: number | null = null;
  private zoomTargetView: string | null = null;
  private srcLong = $state(Number.POSITIVE_INFINITY);
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

  setBaseImage = (element: HTMLImageElement | null): void => {
    this.baseImage = element;
  };

  zoomCycle = (): void => {
    const id = this.assetId;
    if (!id) return;
    if (cachedFaceData(id) === null) {
      void loadFaceData(id).then(() => this.stepZoomTarget(id));
      return;
    }
    this.stepZoomTarget(id);
  };

  private zoomTargets(id: string): ZoomTarget[] {
    const data = cachedFaceData(id);
    const faces = data ? faceTargets(data.faces, viewTransform(this.edits, this.meta)) : [];
    if (faces.length > 0) return faces;
    const sharp = this.baseImage ? sharpestPoint(this.baseImage) : null;
    return [sharp ?? { u: 0.5, v: 0.5 }];
  }

  private stepZoomTarget(id: string): void {
    if (this.assetId !== id) return;
    const targets = this.zoomTargets(id);
    const from = this.viewKeyOf() === this.zoomTargetView ? this.zoomTargetIndex : null;
    const next = nextTargetIndex(from, targets.length);
    this.zoomTargetIndex = next;
    const target = next === null ? null : targets[next];
    const snap = this.viewSnap;
    if (!target) {
      ui.zoomFit();
    } else if (!snap || snap.frame.width <= 0 || ui.zoom <= 0) {
      ui.setZoom(ui.zoomLevel);
    } else {
      const zoom = ui.zoomLevel;
      const pan = panForTarget(target, snap.frame, snap.viewW, snap.viewH, zoom / ui.zoom);
      ui.setView(zoom, pan.panX, pan.panY);
    }
    this.zoomTargetView = this.viewKeyOf();
  }

  private viewKeyOf(): string {
    return `${ui.zoom}:${Math.round(ui.panX)}:${Math.round(ui.panY)}`;
  }

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
      void loadFaceData(id);
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

  retryLensProfile = (): void => {
    if (!this.assetId) return;
    this.fetchLensProfile(this.assetId);
  };

  unload(): void {
    this.flight.cancel();
    this.originalFlight.cancel();
    this.clearView();
    this.viewSnap = null;
    this.zoomTargetIndex = null;
    this.zoomTargetView = null;
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
        await deleteEdits(session.assetId, action, session.hash);
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

  onExport = (opts: ExportOptions): Promise<void> => exportActions.onExport(this, opts);

  retryExport = (): Promise<void> => exportActions.retryExport(this);

  onUploadToImmich = (opts: ImmichExportOptions): Promise<void> =>
    exportActions.onUploadToImmich(this, opts);

  retryUpload = (): Promise<void> => exportActions.retryUpload(this);

  maskCapacityFor = (layerId: string | null): ReturnType<typeof maskCapacity> =>
    maskLayers.maskCapacityFor(this, layerId);

  activeLayer = (): MaskLayer | null => maskLayers.activeLayer(this);

  setActiveLayer = (id: string | null): void => maskLayers.setActiveLayer(this, id);

  activeMaskComponent = (): MaskComponent | null => maskLayers.activeMaskComponent(this);

  setActiveMaskComponent = (id: string | null): void => maskLayers.setActiveMaskComponent(this, id);

  setMaskComponentFeather = (layerId: string, componentId: string, feather: number): void =>
    maskLayers.setMaskComponentFeather(this, layerId, componentId, feather);

  toggleMaskOverlay = (): void => maskLayers.toggleMaskOverlay(this);

  retouchFull = $derived(this.edits.retouch.length >= MAX_RETOUCH_STROKES);

  addRetouchStroke = (stroke: RetouchStroke): Promise<void> =>
    retouch.addRetouchStroke(this, stroke, MAX_RETOUCH_STROKES);

  updateRetouchStroke = (id: string, patch: Partial<RetouchStroke>): void =>
    retouch.updateRetouchStroke(this, id, patch);

  setRetouchStroke = async (
    id: string,
    patch: Partial<RetouchStroke>,
    commit: boolean
  ): Promise<void> => retouch.setRetouchStroke(this, id, patch, commit);

  commitRetouch = (): Promise<void> => retouch.commitRetouch(this);

  removeRetouchStroke = (id: string): Promise<void> => retouch.removeRetouchStroke(this, id);

  toggleRetouchStroke = (id: string): Promise<void> => retouch.toggleRetouchStroke(this, id);

  clearRetouch = (): Promise<void> => retouch.clearRetouch(this);

  previewMaskWeight = (layerId: string): void => maskLayers.previewMaskWeight(this, layerId);

  endMaskPreview = (): void => maskLayers.endMaskPreview(this);

  submitColorPickerPreview = (edits: Edits): void => {
    this.flight.submit({
      edits,
      maxEdge: this.baseEdge(),
      previewMode: 'none',
      purpose: 'color-picker'
    });
  };

  beginColorPicker = (layerId: string, componentId: string): void =>
    maskLayers.beginColorPicker(this, layerId, componentId);

  cancelColorPicker = (): void => maskLayers.cancelColorPicker(this);

  commitColorSample = (sampleRgb: [number, number, number]): Promise<void> =>
    maskLayers.commitColorSample(this, sampleRgb);

  addMaskLayer = (kind: MaskComponentKind = defaultLinear()): Promise<string | null> =>
    maskLayers.addMaskLayer(this, kind);

  removeMaskLayer = (id: string): Promise<void> => maskLayers.removeMaskLayer(this, id);

  duplicateMaskLayer = (id: string): Promise<string | null> =>
    maskLayers.duplicateMaskLayer(this, id);

  reorderMaskLayer = (id: string, toIndex: number): Promise<void> =>
    maskLayers.reorderMaskLayer(this, id, toIndex);

  reorderMaskComponent = (layerId: string, id: string, toIndex: number): Promise<void> =>
    maskLayers.reorderMaskComponent(this, layerId, id, toIndex);

  patchMaskLayer = (id: string, patch: Partial<MaskLayer>, live = true): void =>
    maskLayers.patchMaskLayer(this, id, patch, live);

  toggleMaskLayerEnabled = (id: string): Promise<void> =>
    maskLayers.toggleMaskLayerEnabled(this, id);

  renameMaskLayer = (id: string, name: string): Promise<void> =>
    maskLayers.renameMaskLayer(this, id, name);

  setMaskLayerColor = (id: string, color: string): Promise<void> =>
    maskLayers.setMaskLayerColor(this, id, color);

  setMaskLayerAmount = (id: string, amount: number): void =>
    maskLayers.setMaskLayerAmount(this, id, amount);

  toggleMaskLayerInvert = (id: string): Promise<void> => maskLayers.toggleMaskLayerInvert(this, id);

  setMaskLayerEdit = (id: string, key: MaskedEditKey, value: number): void =>
    maskLayers.setMaskLayerEdit(this, id, key, value);

  resetMaskLayerEdits = (id: string): Promise<void> => maskLayers.resetMaskLayerEdits(this, id);

  beginPolygon = (layerId: string | null, mode: MaskComponentMode = 'add'): void =>
    maskLayers.beginPolygon(this, layerId, mode);

  addPolygonPoint = (point: Vec2f): void => maskLayers.addPolygonPoint(this, point);

  undoPolygonPoint = (): void => maskLayers.undoPolygonPoint(this);

  cancelPolygon = (): void => maskLayers.cancelPolygon(this);

  finishPolygon = (): Promise<void> => maskLayers.finishPolygon(this);

  addMaskComponent = async (
    layerId: string,
    kind: MaskComponentKind,
    mode: MaskComponentMode = 'add'
  ): Promise<string | null> => maskLayers.addMaskComponent(this, layerId, kind, mode);

  removeMaskComponent = (layerId: string, componentId: string): Promise<void> =>
    maskLayers.removeMaskComponent(this, layerId, componentId);

  patchMaskComponent = (
    layerId: string,
    componentId: string,
    patch: Partial<MaskComponent>,
    live = true
  ): void => maskLayers.patchMaskComponent(this, layerId, componentId, patch, live);

  updateMaskComponentKind = (
    layerId: string,
    componentId: string,
    kind: MaskComponentKind,
    live = true
  ): void => maskLayers.updateMaskComponentKind(this, layerId, componentId, kind, live);

  commitMasks = (): Promise<void> => maskLayers.commitMasks(this);

  setBrushTool = (
    patch: Partial<{ size: number; hardness: number; flow: number; mode: 'paint' | 'erase' }>
  ): void => maskLayers.setBrushTool(this, patch);

  setRetouchTool = (
    patch: Partial<{ mode: RetouchMode; size: number; hardness: number; opacity: number }>
  ): void => retouch.setRetouchTool(this, patch);

  setRetouchMode = (mode: RetouchMode): void => retouch.setRetouchMode(this, mode);

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
