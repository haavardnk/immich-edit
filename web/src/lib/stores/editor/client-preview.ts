import { ApiError, NetworkError } from '$lib/api/client';
import { previewModeIsNone, type ProofOptions } from '$lib/api/preview';
import type { ClientRenderer } from '$lib/render/client-renderer';
import { frameMeta } from '$lib/render/frame-meta';
import type { RenderView, RenderedFrame, SourceInfo } from '$lib/render/protocol';
import { renderer } from '$lib/stores/renderer.svelte';
import { scopes } from '$lib/stores/scopes.svelte';
import type { Edits } from '$lib/types/edits';
import { revoke } from '$lib/utils/object-url';
import type { RenderRequest } from '$lib/utils/view-geometry';
import { BASE_SLOT, ClientLane, TILE_SLOT, type ClientJob, type SourceSlot } from './client-lane';
import type { BaseArgs, PreviewCtx, PreviewFrame } from './preview.svelte';

export interface ClientPreviewHooks {
  proof(): ProofOptions;
  baseEdge(): number;
  onBaseIdle(): void;
  onSourceLong(long: number): void;
  onTile(request: RenderRequest, w: number, h: number): void;
  onFallback(): void;
  refreshOriginal(): void;
}

export class ClientPreview {
  private readonly ctx: PreviewCtx;
  private readonly hooks: ClientPreviewHooks;
  private client: ClientRenderer | null = null;
  private base: ClientLane | null = null;
  private tile: ClientLane | null = null;
  private tileRequest: RenderRequest | null = null;
  private serverOnly = false;

  constructor(ctx: PreviewCtx, hooks: ClientPreviewHooks) {
    this.ctx = ctx;
    this.hooks = hooks;
  }

  get busy(): boolean {
    return !!this.base?.busy;
  }

  get serverBound(): boolean {
    return this.serverOnly;
  }

  submitBase(args: BaseArgs): boolean {
    const lane = this.lanes()?.base;
    if (!lane) return false;
    this.ctx.pending = true;
    lane.submit(this.baseJob(args));
    return true;
  }

  submitTile(request: RenderRequest, edits: Edits): boolean {
    const lane = this.lanes()?.tile;
    if (!lane) return false;
    this.tileRequest = request;
    lane.submit(this.tileJob(request, edits));
    return true;
  }

  refreshTile(edits: Edits): boolean {
    if (!this.tileRequest || !this.ctx.viewFrame) return false;
    return this.submitTile(this.tileRequest, edits);
  }

  cancelTile(): void {
    this.tile?.cancel();
    this.tileRequest = null;
    this.ctx.viewFrame?.bitmap.close();
    this.ctx.viewFrame = null;
  }

  dropBase(): void {
    this.ctx.previewFrame?.bitmap.close();
    this.ctx.previewFrame = null;
  }

  reset(): void {
    this.base?.reset();
    this.tile?.reset();
    this.serverOnly = false;
    this.cancelTile();
    this.dropBase();
  }

  private lanes(): { base: ClientLane; tile: ClientLane } | null {
    const client = this.serverOnly ? null : renderer.current;
    if (client !== this.client) {
      this.base?.reset();
      this.tile?.reset();
      this.client = client;
      this.base =
        client &&
        this.lane(client, BASE_SLOT, (job, frame, source, ms) =>
          this.onBase(job, frame, source, ms)
        );
      this.tile = client && this.lane(client, TILE_SLOT, (job, frame) => this.onTile(job, frame));
    }
    return this.base && this.tile ? { base: this.base, tile: this.tile } : null;
  }

  private lane(
    client: ClientRenderer,
    slot: SourceSlot,
    onFrame: (job: ClientJob, frame: RenderedFrame, source: SourceInfo, ms: number) => void
  ): ClientLane {
    return new ClientLane(client, slot, {
      assetId: () => this.ctx.assetId,
      onFrame,
      onError: (err) => this.onError(err),
      onIdle: () => {
        if (slot === BASE_SLOT) this.hooks.onBaseIdle();
      }
    });
  }

  private view(): Omit<RenderView, 'max_edge'> {
    const proof = this.hooks.proof();
    return {
      output_color_space: proof.colorSpace,
      gamut_warn: proof.gamutWarn,
      clip_warn: proof.clipWarn
    };
  }

  private baseJob(args: BaseArgs): ClientJob {
    const plain = previewModeIsNone(args.previewMode);
    return {
      edits: args.edits,
      purpose: args.purpose,
      view: {
        ...this.view(),
        max_edge: this.hooks.baseEdge(),
        preview_mode: args.previewMode,
        histogram: plain,
        scopes: plain && scopes.wants
      }
    };
  }

  private tileJob(request: RenderRequest, edits: Edits): ClientJob {
    return {
      edits,
      request,
      view: { ...this.view(), max_edge: request.maxEdge, roi: request.roi }
    };
  }

  private onBase(job: ClientJob, frame: RenderedFrame, source: SourceInfo, ms: number): void {
    const prev = this.ctx.previewFrame;
    this.ctx.previewFrame = surface(job, frame);
    prev?.bitmap.close();
    if (this.ctx.previewUrl?.startsWith('blob:')) revoke(this.ctx.previewUrl);
    this.ctx.previewUrl = null;
    this.ctx.pending = false;
    renderer.recordRender(ms);
    if (job.purpose === 'color-picker' && this.ctx.colorPicker) {
      this.ctx.colorPicker = { ...this.ctx.colorPicker, ready: true };
    }
    if (!job.view.histogram || !this.ctx.assetId) return;
    this.ctx.meta = frameMeta(this.ctx.assetId, frame, source.is_raw, 'browser');
    const long = Math.max(frame.source_w, frame.source_h);
    if (long > 0) this.hooks.onSourceLong(long);
    scopes.onGrids(frame.scopes ?? null);
    if (this.ctx.splitMode) this.hooks.refreshOriginal();
  }

  private onTile(job: ClientJob, frame: RenderedFrame): void {
    if (!job.request || !job.view.roi || job.request !== this.tileRequest) {
      frame.bitmap.close();
      return;
    }
    const prev = this.ctx.viewFrame;
    this.ctx.viewFrame = surface(job, frame);
    prev?.bitmap.close();
    if (this.ctx.viewUrl?.startsWith('blob:')) revoke(this.ctx.viewUrl);
    this.ctx.viewUrl = null;
    this.ctx.viewRoi = job.view.roi;
    this.ctx.viewNat = { w: frame.width, h: frame.height };
    this.hooks.onTile(job.request, frame.width, frame.height);
  }

  private onError(err: unknown): void {
    if (err instanceof ApiError && err.code === 'superseded') return;
    if (!(err instanceof ApiError || err instanceof NetworkError)) renderer.fail(err);
    this.serverOnly = true;
    this.lanes();
    this.cancelTile();
    this.ctx.pending = false;
    this.hooks.onFallback();
  }
}

function surface(job: ClientJob, frame: RenderedFrame): PreviewFrame {
  return {
    bitmap: frame.bitmap,
    colorSpace: job.view.output_color_space === 'displayp3' ? 'display-p3' : 'srgb'
  };
}
