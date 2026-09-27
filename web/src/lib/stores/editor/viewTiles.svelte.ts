import { livePreview, type ProofOptions } from '$lib/api/preview';
import type { Edits } from '$lib/types/edits';
import { makeObjectUrl, revoke } from '$lib/utils/objectUrl';
import { SingleFlight } from '$lib/utils/singleFlight';
import {
  isFullFrame,
  renderRequest,
  visibleRegion,
  type RenderRequest,
  type Roi
} from '$lib/utils/viewGeometry';
import type { PreviewFrame } from './preview.svelte';
import type { ViewSnapshot } from './previewEdge';

const VIEW_MAX_EDGE = 65535;

export interface ViewTileCtx {
  assetId: string | null;
  edits: Edits;
  previewFrame: PreviewFrame | null;
  viewUrl: string | null;
  viewRoi: Roi | null;
  viewNat: { w: number; h: number } | null;
}

export interface ViewTileHooks {
  proof(): ProofOptions;
  submitClient(request: RenderRequest): boolean;
  cancelClient(): void;
}

export class ViewTiles {
  srcLong = $state(Number.POSITIVE_INFINITY);
  private readonly ctx: ViewTileCtx;
  private readonly hooks: ViewTileHooks;
  private key = '';
  private fullEdge = 0;

  constructor(ctx: ViewTileCtx, hooks: ViewTileHooks) {
    this.ctx = ctx;
    this.hooks = hooks;
  }

  private flight = new SingleFlight<RenderRequest, { url: string; w: number; h: number }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      const { blob } = await livePreview(
        this.ctx.assetId,
        $state.snapshot(this.ctx.edits) as Edits,
        args.maxEdge,
        'none',
        this.hooks.proof(),
        signal,
        'roi',
        isFullFrame(args.roi) ? undefined : args.roi
      );
      const url = makeObjectUrl(blob);
      const decoded = new Image();
      decoded.src = url;
      await decoded.decode().catch(() => undefined);
      if (signal.aborted) {
        revoke(url);
        throw new DOMException('aborted', 'AbortError');
      }
      return { url, w: decoded.naturalWidth, h: decoded.naturalHeight };
    },
    (args, result) => {
      const prev = this.ctx.viewUrl;
      this.ctx.viewUrl = result.url;
      this.ctx.viewRoi = args.roi;
      this.ctx.viewNat = { w: result.w, h: result.h };
      this.land(args, result.w, result.h);
      if (prev?.startsWith('blob:')) revoke(prev);
    },
    () => {
      this.key = '';
    }
  );

  scale(snap: ViewSnapshot | null): number | null {
    if (!snap || !this.ctx.viewRoi || !this.ctx.viewNat) return null;
    const boxLong =
      Math.max(this.ctx.viewRoi[2] * snap.frame.width, this.ctx.viewRoi[3] * snap.frame.height) *
      snap.dpr;
    if (boxLong <= 0) return null;
    return Math.max(this.ctx.viewNat.w, this.ctx.viewNat.h) / boxLong;
  }

  submit(snap: ViewSnapshot): void {
    const visible = visibleRegion(snap.frame, snap.viewW, snap.viewH);
    if (!visible) return;
    const req = renderRequest({
      frame: snap.frame,
      visible,
      dpr: snap.dpr,
      srcLong: this.srcLong,
      serverMaxEdge: VIEW_MAX_EDGE,
      haveRoi: this.ctx.viewRoi,
      haveFullEdge: this.fullEdge
    });
    if (!req) return;
    const drawn = this.ctx.previewFrame?.bitmap;
    if (drawn && req.fullEdge <= Math.max(drawn.width, drawn.height)) return;
    const key = `${req.roi.join(',')}:${req.maxEdge}`;
    if (key === this.key) return;
    this.key = key;
    if (!this.hooks.submitClient(req)) this.flight.submit(req);
  }

  land(request: RenderRequest, w: number, h: number): void {
    const delivered = Math.max(w, h);
    const fraction = request.fullEdge > 0 ? request.maxEdge / request.fullEdge : 1;
    this.fullEdge = Math.round(delivered / fraction);
    if (delivered < request.maxEdge * 0.98) this.srcLong = this.fullEdge;
  }

  clear(): void {
    this.flight.cancel();
    this.hooks.cancelClient();
    if (this.ctx.viewUrl?.startsWith('blob:')) revoke(this.ctx.viewUrl);
    this.ctx.viewUrl = null;
    this.ctx.viewRoi = null;
    this.ctx.viewNat = null;
    this.fullEdge = 0;
    this.key = '';
  }
}
