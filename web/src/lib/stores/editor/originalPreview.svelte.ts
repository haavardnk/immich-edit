import { livePreview, type ProofOptions } from '$lib/api/preview';
import { originalPreviewEdits, type Edits } from '$lib/types/edits';
import { makeObjectUrl, revoke } from '$lib/utils/objectUrl';
import { SingleFlight } from '$lib/utils/singleFlight';

export interface OriginalCtx {
  assetId: string | null;
  edits: Edits;
  splitMode: boolean;
  originalUrl: string | null;
}

export interface OriginalHooks {
  proof(): ProofOptions;
  edge(): number;
}

export class OriginalPreview {
  private readonly ctx: OriginalCtx;
  private readonly hooks: OriginalHooks;
  private edge = 0;
  private geomKey = '';

  constructor(ctx: OriginalCtx, hooks: OriginalHooks) {
    this.ctx = ctx;
    this.hooks = hooks;
  }

  private flight = new SingleFlight<{ edge: number; geomKey: string }, { url: string }>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      const snap = $state.snapshot(this.ctx.edits) as Edits;
      const { blob } = await livePreview(
        this.ctx.assetId,
        originalPreviewEdits(snap),
        args.edge,
        'none',
        this.hooks.proof(),
        signal,
        'original'
      );
      return { url: makeObjectUrl(blob) };
    },
    (args, result) => {
      const prev = this.ctx.originalUrl;
      this.ctx.originalUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.edge = args.edge;
      this.geomKey = args.geomKey;
    },
    () => {}
  );

  refresh(force = false): void {
    if (!this.ctx.splitMode || !this.ctx.assetId) return;
    const edge = this.hooks.edge();
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    const geomKey = JSON.stringify({
      g: snap.geometry,
      l: snap.lens,
      d: snap.color.dcp
    });
    if (!force && this.edge === edge && this.geomKey === geomKey && this.ctx.originalUrl) return;
    this.flight.submit({ edge, geomKey });
  }

  drop(): void {
    this.flight.cancel();
    if (this.ctx.originalUrl?.startsWith('blob:')) revoke(this.ctx.originalUrl);
    this.ctx.originalUrl = null;
    this.edge = 0;
    this.geomKey = '';
  }
}
