import { untrack } from 'svelte';
import { livePreview, maskWeightPreview } from '#lib/api/preview.ts';
import type { Edits } from '#lib/types/edits.ts';
import { SingleFlight } from '#lib/utils/singleFlight.ts';

export type MaskStroke = 'idle' | 'painting' | 'awaiting';

export interface MaskWeightFrame {
  layerId: string;
  componentId: string | null;
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

type WeightArgs = {
  layerId: string;
  componentId: string | null;
  key: string;
  edits: Edits;
  edge: number;
};

export function weightEdits(
  edits: Edits,
  layerId: string,
  componentId: string | null = null
): Edits | null {
  const layer = edits.masks.find((item) => item.id === layerId);
  if (!layer) return null;
  const solo = { ...layer, edits: {}, amount: 1, enabled: true };
  if (componentId === null) return { ...edits, masks: [solo] };
  const component = layer.components.find((item) => item.id === componentId);
  if (!component) return null;
  return {
    ...edits,
    masks: [{ ...solo, invert: false, components: [{ ...component, mode: 'add', enabled: true }] }]
  };
}

export class MaskWeight {
  private readonly ctx: MaskWeightCtx;
  private readonly edge: () => number;
  private layerId: string | null = null;
  private componentId: string | null = null;
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
      if (args.layerId !== this.layerId || args.componentId !== this.componentId) return;
      this.ctx.maskWeight = {
        layerId: args.layerId,
        componentId: args.componentId,
        key: args.key,
        blob
      };
      if (args.key === this.wanted) this.settle();
    },
    () => this.settle()
  );

  sync(layerId: string | null, componentId: string | null = null): void {
    const snap = $state.snapshot(this.ctx.edits) as Edits;
    const edits = layerId && this.ctx.initialised ? weightEdits(snap, layerId, componentId) : null;
    if (!layerId || !edits || !this.ctx.assetId) {
      this.flight.cancel();
      this.layerId = null;
      this.componentId = null;
      this.wanted = '';
      untrack(() => this.settle());
      return;
    }
    const edge = this.edge();
    const key = JSON.stringify({ edits, edge, componentId });
    this.layerId = layerId;
    this.componentId = componentId;
    if (key === this.wanted) return;
    untrack(() => this.want({ layerId, componentId, key, edits, edge }));
  }

  private want(args: WeightArgs): void {
    this.wanted = args.key;
    if (this.ctx.maskWeight?.key === args.key) {
      this.settle();
      return;
    }
    this.flight.submit(args);
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
    this.componentId = null;
    this.wanted = '';
    this.ctx.maskWeight = null;
    this.ctx.maskStroke = 'idle';
  }

  private settle(): void {
    if (this.ctx.maskStroke === 'awaiting') this.ctx.maskStroke = 'idle';
  }
}
