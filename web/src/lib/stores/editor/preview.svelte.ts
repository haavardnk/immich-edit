import { ApiError } from '$lib/api/client';
import {
  getPreviewMeta,
  livePreview,
  maskWeightPreview,
  persistedPreviewUrl,
  previewModeIsNone,
  type PreviewMode,
  type ProofOptions
} from '$lib/api/preview';
import type { ColorSpaceOpt } from '$lib/api/export';
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
import { displayGamutIsWide, previewColorSpace } from '$lib/utils/colorGamut';
import { errorMessage } from '$lib/utils/errors';
import { makeObjectUrl, revoke } from '$lib/utils/objectUrl';
import { SingleFlight } from '$lib/utils/singleFlight';
import type { Roi } from '$lib/utils/viewGeometry';
import type { GeometrySession } from './geometry.svelte';
import { ClientPreview } from './clientPreview';
import { OriginalPreview } from './originalPreview.svelte';
import { baseEdge, dragEdge, fitEdge, LIVE_EDGE, MAX_EDGE, type ViewSnapshot } from './previewEdge';
import { ViewTiles } from './viewTiles.svelte';

const VIEW_DEBOUNCE_MS = 150;

export interface PreviewFrame {
  bitmap: ImageBitmap;
  colorSpace: PredefinedColorSpace;
}

export type BaseArgs = {
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
  viewFrame: PreviewFrame | null;
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
  private viewAfterBase = false;
  private dragging = false;
  private draggedLive = false;
  private previewing = false;
  private deferred: BaseArgs | null = null;
  private lastBase: BaseArgs | null = null;
  private tiles: ViewTiles;
  private original: OriginalPreview;
  private client: ClientPreview;

  constructor(ctx: PreviewCtx) {
    this.ctx = ctx;
    this.tiles = new ViewTiles(ctx, {
      proof: () => this.proofOptions(),
      submitClient: (request) => this.client.submitTile(request, this.snapshotEdits()),
      cancelClient: () => this.client.cancelTile()
    });
    this.original = new OriginalPreview(ctx, {
      proof: () => this.proofOptions(),
      edge: () => baseEdge(this.viewSnap)
    });
    this.client = new ClientPreview(ctx, {
      proof: () => this.proofOptions(),
      baseEdge: () => fitEdge(this.viewSnap),
      onBaseIdle: () => this.baseIdle(),
      onSourceLong: (long) => {
        this.tiles.srcLong = long;
      },
      onTile: (request, w, h) => this.tiles.land(request, w, h),
      onFallback: () => {
        if (this.lastBase) this.flight.submit(this.lastBase);
      },
      refreshOriginal: () => this.original.refresh()
    });
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
        if (this.ctx.splitMode) this.original.refresh();
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

  get sourceLong(): number {
    return this.tiles.srcLong;
  }

  get snapshot(): ViewSnapshot | null {
    return this.viewSnap;
  }

  get viewScale(): number | null {
    return this.tiles.scale(this.viewSnap);
  }

  live(): void {
    if (!this.ctx.initialised) return;
    this.cancelRelease();
    if (this.viewBlocked() || !this.client.refreshTile(this.snapshotEdits())) this.clearView();
    if (this.dragging) {
      this.draggedLive = true;
      this.submitBase(this.settledArgs(dragEdge(this.viewSnap)));
      return;
    }
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap)));
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
    this.submitBase({
      edits: this.snapshotEdits(),
      maxEdge: this.dragging ? dragEdge(this.viewSnap) : LIVE_EDGE,
      previewMode: mode
    });
  }

  refreshBase(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap)));
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
      this.submitBase(this.settledArgs(baseEdge(this.viewSnap)));
      this.original.refresh();
    } else {
      this.original.drop();
      this.scheduleIdle();
    }
  }

  reproof(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.clearView();
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap)));
    this.scheduleIdle();
    if (this.ctx.splitMode) this.original.refresh(true);
  }

  onViewChange(snap: ViewSnapshot): void {
    this.viewSnap = snap;
    if (this.dragging || this.viewBlocked()) return;
    this.scheduleIdle();
  }

  clearView(): void {
    this.tiles.clear();
    this.viewAfterBase = false;
    if (this.idleTimer) {
      clearTimeout(this.idleTimer);
      this.idleTimer = null;
    }
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
    this.client.reset();
    this.deferred = null;
    this.lastBase = null;
    this.cancelRelease();
    this.dragging = false;
    this.draggedLive = false;
    this.previewing = false;
    this.clearView();
    this.viewSnap = null;
    this.tiles.srcLong = Number.POSITIVE_INFINITY;
    if (this.ctx.previewUrl?.startsWith('blob:')) revoke(this.ctx.previewUrl);
    this.ctx.previewUrl = null;
    this.dropFrame();
    this.original.drop();
    this.ctx.splitMode = false;
  }

  private snapshotEdits(): Edits {
    return $state.snapshot(this.ctx.edits) as Edits;
  }

  private settledArgs(maxEdge: number): BaseArgs {
    const edits = this.snapshotEdits();
    const section = this.ctx.bypassedSection;
    const layer = this.ctx.maskPreviewLayerId;
    if (this.ctx.colorPicker) {
      return {
        edits: { ...edits, masks: [] },
        maxEdge,
        previewMode: 'none',
        purpose: 'color-picker'
      };
    }
    if (this.ctx.showingOriginal) {
      return { edits: originalPreviewEdits(edits), maxEdge, previewMode: 'none' };
    }
    if (section) return { edits: neutraliseSection(edits, section), maxEdge, previewMode: 'none' };
    return { edits, maxEdge, previewMode: layer ? maskWeightPreview(layer) : 'none' };
  }

  private submitBase(args: BaseArgs): void {
    this.previewing = !previewModeIsNone(args.previewMode);
    this.lastBase = args;
    if (!this.client.serverBound && renderer.undecided) {
      this.deferred = args;
      void renderer.start().then(() => {
        const next = this.deferred;
        this.deferred = null;
        if (next) this.submitBase(next);
      });
      return;
    }
    if (!this.client.submitBase(args)) this.flight.submit(args);
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
    this.client.dropBase();
  }

  private fireIdle(): void {
    if (!this.ctx.initialised) return;
    if (this.ctx.splitMode) {
      this.submitBase(this.settledArgs(baseEdge(this.viewSnap)));
      return;
    }
    if (this.viewBlocked()) return;
    if (this.flight.busy || this.client.busy) {
      this.viewAfterBase = true;
      return;
    }
    if (this.viewSnap) this.tiles.submit(this.viewSnap);
  }

  private cancelRelease(): void {
    if (!this.releaseTimer) return;
    clearTimeout(this.releaseTimer);
    this.releaseTimer = null;
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
      this.previewing ||
      !!this.ctx.colorPicker
    );
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
      if (Number.isFinite(long) && long > 0) this.tiles.srcLong = long;
      scopes.onMeta(this.ctx.assetId, metaId, this.ctx.meta.has_scopes);
    } catch {
      this.ctx.meta = null;
    }
  }
}
