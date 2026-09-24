import { ApiError, NetworkError } from '$lib/api/client';
import {
  getPreviewMeta,
  livePreview,
  persistedPreviewUrl,
  previewModeIsNone,
  type PreviewMode,
  type ProofOptions
} from '$lib/api/preview';
import type { ColorSpaceOpt } from '$lib/api/export';
import type { ClientRenderer } from '$lib/render/client-renderer';
import { frameMeta } from '$lib/render/frame-meta';
import type { RenderedFrame, SourceInfo } from '$lib/render/protocol';
import { renderer } from '$lib/stores/renderer.svelte';
import { scopes } from '$lib/stores/scopes.svelte';
import { ui } from '$lib/stores/ui.svelte';
import {
  neutraliseSection,
  originalPreviewEdits,
  type DevelopSection,
  type Edits
} from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';
import { displayGamutIsWide, previewColorSpace } from '$lib/utils/color-gamut';
import { errorMessage } from '$lib/utils/errors';
import { makeObjectUrl, revoke } from '$lib/utils/object-url';
import { SingleFlight } from '$lib/utils/single-flight';
import {
  isFullFrame,
  renderRequest,
  visibleRegion,
  type Rect,
  type RenderRequest,
  type Roi
} from '$lib/utils/view-geometry';
import type { GeometrySession } from './geometry.svelte';
import { ClientLane, type ClientJob } from './client-lane';

const LIVE_EDGE = 1600;
const MAX_EDGE = 4096;
const VIEW_MAX_EDGE = 65535;
const VIEW_DEBOUNCE_MS = 150;

export type ViewSnapshot = { frame: Rect; viewW: number; viewH: number; dpr: number };

export interface PreviewFrame {
  bitmap: ImageBitmap;
  colorSpace: PredefinedColorSpace;
}

type BaseArgs = {
  edits: Edits;
  maxEdge: number;
  previewMode: PreviewMode;
  purpose?: 'color-picker';
};

export interface PreviewCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  meta: PreviewMeta | null;
  previewUrl: string | null;
  previewFrame: PreviewFrame | null;
  originalUrl: string | null;
  viewUrl: string | null;
  viewRoi: Roi | null;
  viewNat: { w: number; h: number } | null;
  pending: boolean;
  error: string | null;
  splitMode: boolean;
  showingOriginal: boolean;
  bypassedSection: DevelopSection | null;
  geometrySession: GeometrySession | null;
  maskPreviewLayerId: string | null;
  colorPicker: { layerId: string; componentId: string; ready: boolean } | null;
  proofSpace: ColorSpaceOpt;
  gamutWarn: boolean;
}

export class PreviewEngine {
  private ctx: PreviewCtx;
  private idleTimer: ReturnType<typeof setTimeout> | null = null;
  private releaseTimer: ReturnType<typeof setTimeout> | null = null;
  private viewSnap = $state<ViewSnapshot | null>(null);
  private viewKey = '';
  private viewFullEdge = 0;
  private viewAfterBase = false;
  private dragging = false;
  private draggedLive = false;
  private srcLong = $state(Number.POSITIVE_INFINITY);
  private originalEdge = 0;
  private originalGeomKey = '';
  private lane: ClientLane | null = null;
  private laneClient: ClientRenderer | null = null;
  private deferred: BaseArgs | null = null;
  private lastBase: BaseArgs | null = null;
  private serverOnly = false;

  constructor(ctx: PreviewCtx) {
    this.ctx = ctx;
  }

  private flight = new SingleFlight<BaseArgs, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      this.ctx.pending = true;
      const { blob, metaId } = await livePreview(
        this.ctx.assetId,
        args.edits,
        args.maxEdge,
        args.previewMode,
        this.proofOptions(),
        signal,
        'base',
        undefined,
        scopes.wants
      );
      return { url: makeObjectUrl(blob), metaId };
    },
    (args, result) => {
      const prev = this.ctx.previewUrl;
      this.ctx.previewUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.dropFrame();
      this.ctx.pending = false;
      this.markPickerReady(args.purpose);
      if (previewModeIsNone(args.previewMode)) {
        if (result.metaId) void this.loadMeta(result.metaId);
        if (this.ctx.splitMode) this.refreshOriginal();
      }
    },
    (err) => {
      this.ctx.pending = false;
      if (err instanceof DOMException && err.name === 'AbortError') return;
      if (err instanceof ApiError && err.code === 'superseded') return;
      this.ctx.colorPicker = null;
      this.ctx.error = errorMessage(err);
    },
    () => this.baseIdle()
  );

  private originalFlight = new SingleFlight<{ edge: number; geomKey: string }, { url: string }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      const snap = $state.snapshot(this.ctx.edits) as Edits;
      const edits = originalPreviewEdits(snap);
      const { blob } = await livePreview(
        this.ctx.assetId,
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
      const prev = this.ctx.originalUrl;
      this.ctx.originalUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.originalEdge = args.edge;
      this.originalGeomKey = args.geomKey;
    },
    () => {}
  );

  private viewFlight = new SingleFlight<RenderRequest, { url: string; w: number; h: number }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      const { blob } = await livePreview(
        this.ctx.assetId,
        $state.snapshot(this.ctx.edits) as Edits,
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
      const prev = this.ctx.viewUrl;
      this.ctx.viewUrl = result.url;
      this.ctx.viewRoi = args.roi;
      this.ctx.viewNat = { w: result.w, h: result.h };
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

  get sourceLong(): number {
    return this.srcLong;
  }

  get snapshot(): ViewSnapshot | null {
    return this.viewSnap;
  }

  get viewScale(): number | null {
    const snap = this.viewSnap;
    if (!snap || !this.ctx.viewRoi || !this.ctx.viewNat) return null;
    const boxLong =
      Math.max(this.ctx.viewRoi[2] * snap.frame.width, this.ctx.viewRoi[3] * snap.frame.height) *
      snap.dpr;
    if (boxLong <= 0) return null;
    return Math.max(this.ctx.viewNat.w, this.ctx.viewNat.h) / boxLong;
  }

  live(): void {
    if (!this.ctx.initialised) return;
    this.cancelRelease();
    this.clearView();
    if (this.dragging) {
      this.draggedLive = true;
      this.submitEdits(this.dragEdge(), 'none');
      return;
    }
    this.submitEdits(this.baseEdge(), 'none');
    this.scheduleIdle();
  }

  beginDrag(): void {
    this.dragging = true;
    this.draggedLive = false;
  }

  endDrag(): void {
    if (!this.dragging) return;
    this.dragging = false;
    if (!this.draggedLive) {
      if (!this.viewBlocked()) this.scheduleIdle();
      return;
    }
    this.draggedLive = false;
    this.releaseTimer = setTimeout(() => {
      this.releaseTimer = null;
      this.live();
    }, 0);
  }

  preview(mode: PreviewMode): void {
    if (!this.ctx.initialised) return;
    this.clearView();
    this.submitEdits(this.dragging ? this.dragEdge() : LIVE_EDGE, mode);
  }

  showOriginal(): void {
    if (!this.ctx.initialised) return;
    this.clearView();
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    this.submitBase({
      edits: originalPreviewEdits(snap),
      maxEdge: this.baseEdge(),
      previewMode: 'none'
    });
  }

  bypassSection(section: DevelopSection): void {
    if (!this.ctx.initialised) return;
    this.clearView();
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    this.submitBase({
      edits: neutraliseSection(snap, section),
      maxEdge: this.baseEdge(),
      previewMode: 'none'
    });
  }

  refreshBase(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.submitEdits(this.baseEdge(), 'none');
  }

  submitColorPicker(edits: Edits): void {
    this.submitBase({
      edits,
      maxEdge: this.baseEdge(),
      previewMode: 'none',
      purpose: 'color-picker'
    });
  }

  loadPersisted(): void {
    if (!this.ctx.assetId) return;
    const prev = this.ctx.previewUrl;
    this.ctx.previewUrl =
      persistedPreviewUrl(this.ctx.assetId, MAX_EDGE, ui.clipWarn) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
    this.dropFrame();
  }

  toggleSplit(): void {
    this.clearView();
    this.ctx.splitMode = !this.ctx.splitMode;
    if (this.ctx.splitMode) {
      this.submitEdits(this.baseEdge(), 'none');
      this.refreshOriginal();
    } else {
      this.dropOriginal();
      this.scheduleIdle();
    }
  }

  reproof(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.clearView();
    this.submitEdits(this.baseEdge(), 'none');
    this.scheduleIdle();
    if (this.ctx.splitMode) this.refreshOriginal(true);
  }

  onViewChange(snap: ViewSnapshot): void {
    this.viewSnap = snap;
    if (this.dragging || this.viewBlocked()) return;
    this.scheduleIdle();
  }

  clearView(): void {
    this.viewFlight.cancel();
    this.viewAfterBase = false;
    if (this.idleTimer) {
      clearTimeout(this.idleTimer);
      this.idleTimer = null;
    }
    if (this.ctx.viewUrl?.startsWith('blob:')) revoke(this.ctx.viewUrl);
    this.ctx.viewUrl = null;
    this.ctx.viewRoi = null;
    this.ctx.viewNat = null;
    this.viewFullEdge = 0;
    this.viewKey = '';
  }

  scheduleIdle(): void {
    if (this.idleTimer) clearTimeout(this.idleTimer);
    this.idleTimer = setTimeout(() => {
      this.idleTimer = null;
      this.fireIdle();
    }, VIEW_DEBOUNCE_MS);
  }

  reset(): void {
    this.flight.cancel();
    this.lane?.reset();
    this.deferred = null;
    this.lastBase = null;
    this.serverOnly = false;
    this.originalFlight.cancel();
    this.cancelRelease();
    this.dragging = false;
    this.draggedLive = false;
    this.clearView();
    this.viewSnap = null;
    this.srcLong = Number.POSITIVE_INFINITY;
    if (this.ctx.previewUrl?.startsWith('blob:')) revoke(this.ctx.previewUrl);
    this.ctx.previewUrl = null;
    this.dropFrame();
    this.dropOriginal();
    this.ctx.splitMode = false;
  }

  private submitEdits(maxEdge: number, previewMode: PreviewMode): void {
    this.submitBase({ edits: $state.snapshot(this.ctx.edits) as Edits, maxEdge, previewMode });
  }

  private submitBase(args: BaseArgs): void {
    this.lastBase = args;
    if (!this.serverOnly && renderer.undecided) {
      this.deferred = args;
      void renderer.start().then(() => {
        const next = this.deferred;
        this.deferred = null;
        if (next) this.submitBase(next);
      });
      return;
    }
    const lane = this.clientLane();
    if (!lane) {
      this.flight.submit(args);
      return;
    }
    this.ctx.pending = true;
    lane.submit(this.clientJob(args));
  }

  private clientLane(): ClientLane | null {
    const client = this.serverOnly ? null : renderer.current;
    if (this.lane && this.laneClient === client) return this.lane;
    this.lane?.reset();
    this.laneClient = client;
    this.lane = client
      ? new ClientLane(client, {
          assetId: () => this.ctx.assetId,
          onFrame: (job, frame, source, ms) => this.onClientFrame(job, frame, source, ms),
          onError: (err) => this.onClientError(err),
          onIdle: () => this.baseIdle()
        })
      : null;
    return this.lane;
  }

  private clientJob(args: BaseArgs): ClientJob {
    const proof = this.proofOptions();
    const plain = previewModeIsNone(args.previewMode);
    return {
      edits: args.edits,
      purpose: args.purpose,
      view: {
        max_edge: this.baseEdge(),
        output_color_space: proof.colorSpace,
        preview_mode: args.previewMode,
        gamut_warn: proof.gamutWarn,
        clip_warn: proof.clipWarn,
        histogram: plain,
        scopes: plain && scopes.wants
      }
    };
  }

  private onClientFrame(
    job: ClientJob,
    frame: RenderedFrame,
    source: SourceInfo,
    ms: number
  ): void {
    const prev = this.ctx.previewFrame;
    this.ctx.previewFrame = {
      bitmap: frame.bitmap,
      colorSpace: job.view.output_color_space === 'displayp3' ? 'display-p3' : 'srgb'
    };
    prev?.bitmap.close();
    if (this.ctx.previewUrl?.startsWith('blob:')) revoke(this.ctx.previewUrl);
    this.ctx.previewUrl = null;
    this.ctx.pending = false;
    renderer.recordRender(ms);
    this.markPickerReady(job.purpose);
    if (!job.view.histogram || !this.ctx.assetId) return;
    this.ctx.meta = frameMeta(this.ctx.assetId, frame, source.is_raw, 'browser');
    const long = Math.max(frame.source_w, frame.source_h);
    if (long > 0) this.srcLong = long;
    scopes.onGrids(frame.scopes ?? null);
    if (this.ctx.splitMode) this.refreshOriginal();
  }

  private onClientError(err: unknown): void {
    if (err instanceof ApiError && err.code === 'superseded') return;
    if (!(err instanceof ApiError || err instanceof NetworkError)) renderer.fail(err);
    this.serverOnly = true;
    this.clientLane();
    this.ctx.pending = false;
    if (this.lastBase) this.flight.submit(this.lastBase);
  }

  private markPickerReady(purpose: BaseArgs['purpose']): void {
    if (purpose === 'color-picker' && this.ctx.colorPicker) {
      this.ctx.colorPicker = { ...this.ctx.colorPicker, ready: true };
    }
  }

  private baseIdle(): void {
    if (!this.viewAfterBase) return;
    this.viewAfterBase = false;
    this.fireIdle();
  }

  private dropFrame(): void {
    this.ctx.previewFrame?.bitmap.close();
    this.ctx.previewFrame = null;
  }

  private fireIdle(): void {
    if (!this.ctx.initialised) return;
    if (this.ctx.splitMode) {
      this.submitEdits(this.baseEdge(), 'none');
      return;
    }
    if (this.viewBlocked()) return;
    if (this.flight.busy || this.lane?.busy) {
      this.viewAfterBase = true;
      return;
    }
    this.submitView();
  }

  private cancelRelease(): void {
    if (!this.releaseTimer) return;
    clearTimeout(this.releaseTimer);
    this.releaseTimer = null;
  }

  private dropOriginal(): void {
    this.originalFlight.cancel();
    if (this.ctx.originalUrl?.startsWith('blob:')) revoke(this.ctx.originalUrl);
    this.ctx.originalUrl = null;
    this.originalEdge = 0;
    this.originalGeomKey = '';
  }

  private refreshOriginal(force = false): void {
    if (!this.ctx.splitMode || !this.ctx.assetId) return;
    const edge = this.baseEdge();
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    const geomKey = JSON.stringify({
      g: snap.geometry,
      l: snap.lens,
      d: snap.color.dcp
    });
    if (
      !force &&
      this.originalEdge === edge &&
      this.originalGeomKey === geomKey &&
      this.ctx.originalUrl
    )
      return;
    this.originalFlight.submit({ edge, geomKey });
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
      haveRoi: this.ctx.viewRoi,
      haveFullEdge: this.viewFullEdge
    });
    if (!req) return;
    const drawn = this.ctx.previewFrame?.bitmap;
    if (drawn && req.fullEdge <= Math.max(drawn.width, drawn.height)) return;
    const key = `${req.roi.join(',')}:${req.maxEdge}`;
    if (key === this.viewKey) return;
    this.viewKey = key;
    this.viewFlight.submit(req);
  }

  private viewBlocked(): boolean {
    return (
      !this.ctx.initialised ||
      !this.ctx.assetId ||
      this.ctx.splitMode ||
      this.ctx.showingOriginal ||
      !!this.ctx.bypassedSection ||
      !!this.ctx.geometrySession ||
      !!this.ctx.maskPreviewLayerId ||
      !!this.ctx.colorPicker
    );
  }

  private baseEdge(): number {
    const snap = this.viewSnap;
    if (!snap) return LIVE_EDGE;
    const long = Math.round(Math.max(snap.frame.width, snap.frame.height) * snap.dpr);
    return Math.max(LIVE_EDGE, Math.min(MAX_EDGE, long));
  }

  private dragEdge(): number {
    const long = Math.round(Math.max(this.viewSnap?.viewW ?? 0, this.viewSnap?.viewH ?? 0));
    return long > 0 ? Math.min(LIVE_EDGE, long) : LIVE_EDGE;
  }

  private proofOptions(): ProofOptions {
    return {
      colorSpace: previewColorSpace(this.ctx.proofSpace, this.ctx.gamutWarn, displayGamutIsWide()),
      gamutWarn: this.ctx.gamutWarn,
      clipWarn: ui.clipWarn
    };
  }

  private async loadMeta(metaId: string): Promise<void> {
    if (!this.ctx.assetId) return;
    try {
      this.ctx.meta = await getPreviewMeta(this.ctx.assetId, metaId);
      const long = Math.max(this.ctx.meta.source_w, this.ctx.meta.source_h);
      if (Number.isFinite(long) && long > 0) this.srcLong = long;
      scopes.onMeta(this.ctx.assetId, metaId, this.ctx.meta.has_scopes);
    } catch {
      this.ctx.meta = null;
    }
  }
}
