import { untrack } from 'svelte';
import { livePreview, maskWeightPreview } from '$lib/api/preview';
import type { Edits } from '$lib/types/edits';
import { SingleFlight } from '$lib/utils/singleFlight';

export type MaskStroke = 'idle' | 'painting' | 'awaiting';

export interface MaskWeightFrame {
  layerId: string;
  key: string;
  blob: Blob;
}

export interface MaskWeightCtx {
  assetId: string | null;
  initialised: boolean;
  edits: Edits;
  maskWeight: MaskWeightFrame | null;
  maskStroke: MaskStroke;
}

type WeightArgs = { layerId: string; key: string; edits: Edits; edge: number };

export function weightEdits(edits: Edits, layerId: string): Edits | null {
  const layer = edits.masks.find((item) => item.id === layerId);
  if (!layer) return null;
  return { ...edits, masks: [{ ...layer, edits: {}, amount: 1, enabled: true }] };
}

export class MaskWeight {
  private readonly ctx: MaskWeightCtx;
  private readonly edge: () => number;
  private layerId: string | null = null;
  private wanted = '';

  constructor(ctx: MaskWeightCtx, edge: () => number) {
    this.ctx = ctx;
    this.edge = edge;
  }

  private flight = new SingleFlight<WeightArgs, Blob>(
    async (args, signal) => {
      if (!this.ctx.assetId) throw new Error('no asset');
      const { blob } = await livePreview(
        this.ctx.assetId,
        args.edits,
        args.edge,
        maskWeightPreview(args.layerId),
        undefined,
        signal,
        'weight'
      );
      return blob;
    },
    (args, blob) => {
      if (args.layerId !== this.layerId) return;
      this.ctx.maskWeight = { layerId: args.layerId, key: args.key, blob };
      if (args.key === this.wanted) this.settle();
    },
    () => this.settle()
  );

  sync(layerId: string | null): void {
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    const edits = layerId && this.ctx.initialised ? weightEdits(snap, layerId) : null;
    if (!layerId || !edits || !this.ctx.assetId) {
      this.flight.cancel();
      this.layerId = null;
      this.wanted = '';
      untrack(() => this.settle());
      return;
    }
    const edge = this.edge();
    const key = JSON.stringify({ edits, edge });
    this.layerId = layerId;
    if (key === this.wanted) return;
    untrack(() => this.want(layerId, key, edits, edge));
  }

  private want(layerId: string, key: string, edits: Edits, edge: number): void {
    this.wanted = key;
    if (this.ctx.maskWeight?.key === key) {
      this.settle();
      return;
    }
    this.flight.submit({ layerId, key, edits, edge });
  }

  endStroke(): void {
    if (this.ctx.maskStroke !== 'painting') return;
    if (!this.wanted || this.ctx.maskWeight?.key === this.wanted) {
      this.ctx.maskStroke = 'idle';
      return;
    }
    this.ctx.maskStroke = 'awaiting';
  }

  drop(): void {
    this.flight.cancel();
    this.layerId = null;
    this.wanted = '';
    this.ctx.maskWeight = null;
    this.ctx.maskStroke = 'idle';
  }

  private settle(): void {
    if (this.ctx.maskStroke === 'awaiting') this.ctx.maskStroke = 'idle';
  }
}
