import { ApiError } from '#lib/api/client.ts';
import {
  getPreviewMeta,
  livePreview,
  persistedPreviewUrl,
  previewModeIsNone,
  type PreviewMode,
  type ProofOptions
} from '#lib/api/preview.ts';
import type { ColorSpaceOpt } from '#lib/api/export.ts';
import { renderer } from '#lib/stores/renderer.svelte.ts';
import { scopes } from '#lib/stores/scopes.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { DevelopSection, Edits } from '#lib/types/edits.ts';
import type { PreviewMeta } from '#lib/types/preview.ts';
import { displayGamutIsWide, previewColorSpace } from '#lib/utils/colorGamut.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
import { SingleFlight } from '#lib/utils/singleFlight.ts';
import type { Roi } from '#lib/utils/viewGeometry.ts';
import type { GeometrySession } from './geometry.svelte';
import type { EditsPreview, EditsPreviewOwner } from '#lib/stores/editor/editsPreview.ts';
import { ClientPreview } from './clientPreview';
import { displayedEdits } from './displayedEdits';
import { MaskWeight, type MaskWeightCtx } from './maskWeight.svelte';
import { CompareLeft, type CompareLeftCtx } from '#lib/stores/editor/compareLeft.svelte.ts';
import {
  baseEdge,
  dragEdge,
  fitEdge,
  largestEdge,
  windowEdge,
  type ViewSnapshot
} from './previewEdge';
import { budgetFor, type RenderBudget } from '#lib/render/budget.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import { measureServerTick } from './serverTick';
import { ViewTiles } from './viewTiles.svelte';
import type { RangePicker } from '#lib/stores/editor/rangePicker.ts';

const VIEW_DEBOUNCE_MS = 150;

function pageHidden(): boolean {
  return typeof document !== 'undefined' && document.visibilityState === 'hidden';
}

export interface PreviewFrame {
  bitmap: ImageBitmap;
  colorSpace: PredefinedColorSpace;
}

export type BaseArgs = {
  edits: Edits;
  maxEdge: number;
  previewMode: PreviewMode;
  purpose?: 'range-picker';
};

export interface PreviewCtx extends MaskWeightCtx, CompareLeftCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  meta: PreviewMeta | null;
  previewUrl: string | null;
  previewFrame: PreviewFrame | null;
  viewUrl: string | null;
  viewFrame: PreviewFrame | null;
  viewRoi: Roi | null;
  viewNat: { w: number; h: number } | null;
  pending: boolean;
  error: string | null;
  showingOriginal: boolean;
  bypassedSection: DevelopSection | null;
  readonly isRaw: boolean;
  editsPreview: EditsPreview | null;
  geometrySession: GeometrySession | null;
  rangePicker: RangePicker | null;
  proofSpace: ColorSpaceOpt;
  gamutWarn: boolean;
  readonly sourceRevision: string;
}

export class PreviewEngine {
  private ctx: PreviewCtx;
  private idleTimer: ReturnType<typeof setTimeout> | null = null;
  private releaseTimer: ReturnType<typeof setTimeout> | null = null;
  private viewSnap = $state<ViewSnapshot | null>(null);
  private viewAfterBase = false;
  private dragging = false;
  private dragShown = $state(false);
  private draggedLive = false;
  private previewing = false;
  private deferred: BaseArgs | null = null;
  private lastBase: BaseArgs | null = null;
  private tiles: ViewTiles;
  private left: CompareLeft;
  private weight: MaskWeight;
  private client: ClientPreview;

  constructor(ctx: PreviewCtx) {
    this.ctx = ctx;
    this.tiles = new ViewTiles(ctx, {
      proof: () => this.proofOptions(),
      submitClient: (request) => this.client.submitTile(request, this.snapshotEdits()),
      cancelClient: () => this.client.cancelTile()
    });
    this.left = new CompareLeft(ctx, {
      proof: () => this.proofOptions(),
      edge: () => baseEdge(this.viewSnap, this.budget)
    });
    this.weight = new MaskWeight(ctx, () => baseEdge(this.viewSnap, this.budget));
    this.client = new ClientPreview(ctx, {
      proof: () => this.proofOptions(),
      baseEdge: () => fitEdge(this.viewSnap, this.budget),
      onBaseIdle: () => this.baseIdle(),
      onSourceLong: (long) => {
        this.tiles.srcLong = long;
      },
      onTile: (request, w, h) => this.tiles.land(request, w, h),
      onFallback: () => {
        if (this.lastBase) this.flight.submit(this.lastBase);
      },
      refreshLeft: () => this.left.refresh()
    });
    if (typeof document !== 'undefined') {
      document.addEventListener('visibilitychange', () => this.resume());
    }
  }

  private get budget(): RenderBudget {
    return budgetFor(layout.mode);
  }

  private resume(): void {
    const next = this.deferred;
    if (pageHidden() || !next) return;
    this.deferred = null;
    this.submitBase(next);
  }

  private flight = new SingleFlight<BaseArgs, { url: string; metaId: string | null }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      this.ctx.pending = true;
      const { blob, metaId, timing } = await livePreview(
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
      if (timing && renderer.measuresServer && previewModeIsNone(args.previewMode)) {
        void measureServerTick(blob, timing).then((sample) => {
          if (sample) renderer.recordServer(sample);
        });
      }
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
        this.left.refresh();
      }
    },
    (err) => {
      this.ctx.pending = false;
      if (err instanceof DOMException && err.name === 'AbortError') return;
      if (err instanceof ApiError && err.code === 'superseded') return;
      this.ctx.rangePicker = null;
      this.ctx.error = errorMessage(err);
    },
    () => this.baseIdle()
  );

  get sourceLong(): number {
    return this.tiles.srcLong;
  }

  get dragActive(): boolean {
    return this.dragShown;
  }

  get viewScale(): number | null {
    return this.tiles.scale(this.viewSnap);
  }

  syncWeight(layerId: string | null, componentId: string | null = null): void {
    this.weight.sync(layerId, componentId);
  }

  endWeightStroke(): void {
    this.weight.endStroke();
  }

  live(): void {
    if (!this.ctx.initialised) return;
    this.cancelRelease();
    if (this.viewBlocked() || !this.client.refreshTile(this.snapshotEdits())) this.clearView();
    if (this.dragging) {
      this.draggedLive = true;
      this.submitBase(this.settledArgs(dragEdge(this.viewSnap, this.budget)));
      return;
    }
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap, this.budget)));
    this.scheduleIdle();
  }

  showEdits(preview: EditsPreview): void {
    this.ctx.editsPreview = preview;
    this.live();
  }

  endEdits(owner: EditsPreviewOwner): void {
    if (this.ctx.editsPreview?.owner !== owner) return;
    this.ctx.editsPreview = null;
    this.live();
  }

  beginDrag(): void {
    this.dragging = true;
    this.dragShown = true;
    this.draggedLive = false;
  }

  endDrag(): void {
    if (!this.dragging) return;
    this.dragging = false;
    this.dragShown = false;
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
      maxEdge: this.dragging ? dragEdge(this.viewSnap, this.budget) : this.budget.liveEdge,
      previewMode: mode
    });
  }

  refreshBase(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap, this.budget)));
  }

  loadPersisted(): void {
    if (!this.ctx.assetId) return;
    const prev = this.ctx.previewUrl;
    this.ctx.previewUrl =
      persistedPreviewUrl(this.ctx.assetId, this.budget.maxEdge, ui.clipWarn) + `&_=${Date.now()}`;
    if (prev?.startsWith('blob:')) revoke(prev);
    this.dropFrame();
  }

  compareChanged(wasOn: boolean): void {
    this.clearView();
    if (this.ctx.compare.mode === 'off') {
      this.left.drop();
      this.scheduleIdle();
      return;
    }
    if (!wasOn) this.submitBase(this.settledArgs(baseEdge(this.viewSnap, this.budget)));
    this.left.refresh();
  }

  reproof(): void {
    if (!this.ctx.initialised || !this.ctx.assetId) return;
    this.clearView();
    this.submitBase(this.settledArgs(baseEdge(this.viewSnap, this.budget)));
    this.scheduleIdle();
    this.left.refresh(true);
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
    this.dragShown = false;
    this.draggedLive = false;
    this.previewing = false;
    this.clearView();
    this.viewSnap = null;
    this.tiles.srcLong = Number.POSITIVE_INFINITY;
    if (this.ctx.previewUrl?.startsWith('blob:')) revoke(this.ctx.previewUrl);
    this.ctx.previewUrl = null;
    this.dropFrame();
    this.left.drop();
    this.weight.drop();
  }

  private snapshotEdits(): Edits {
    return $state.snapshot(this.ctx.edits) as Edits;
  }

  private settledArgs(maxEdge: number): BaseArgs {
    const edits = displayedEdits(this.ctx, this.snapshotEdits());
    if (this.ctx.rangePicker) {
      return { edits, maxEdge, previewMode: 'none', purpose: 'range-picker' };
    }
    return { edits, maxEdge, previewMode: 'none' };
  }

  private submitBase(args: BaseArgs): void {
    this.previewing = !previewModeIsNone(args.previewMode);
    this.lastBase = args;
    if (pageHidden()) {
      this.deferred = args;
      return;
    }
    if (this.client.unbound) {
      const binding = renderer.bind({
        benchmarkEdge: fitEdge(this.viewSnap, this.budget),
        maxEdge: Math.max(largestEdge(this.viewSnap, this.budget), windowEdge(this.budget))
      });
      if (binding.kind === 'wait') {
        this.deferred = args;
        void binding.ready.then(() => {
          const next = this.deferred;
          this.deferred = null;
          if (next) this.submitBase(next);
        });
        return;
      }
      this.client.bind(binding.kind === 'browser' ? binding.client : null);
    }
    if (!this.client.submitBase(args)) this.flight.submit(args);
  }

  private markPickerReady(purpose: BaseArgs['purpose']): void {
    if (purpose === 'range-picker' && this.ctx.rangePicker) {
      this.ctx.rangePicker = { ...this.ctx.rangePicker, ready: true };
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
    if (this.ctx.compare.mode !== 'off') {
      this.submitBase(this.settledArgs(baseEdge(this.viewSnap, this.budget)));
      return;
    }
    if (this.viewBlocked()) return;
    if (this.flight.busy || this.client.busy || this.deferred) {
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
      this.ctx.compare.mode !== 'off' ||
      this.ctx.showingOriginal ||
      !!this.ctx.bypassedSection ||
      !!this.ctx.editsPreview ||
      !!this.ctx.geometrySession ||
      this.previewing ||
      !!this.ctx.rangePicker
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
