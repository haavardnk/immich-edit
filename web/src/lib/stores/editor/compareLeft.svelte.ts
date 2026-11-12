import { getEdits } from '#lib/api/edits.ts';
import { livePreview, type ProofOptions } from '#lib/api/preview.ts';
import type { EditSnapshot } from '#lib/api/snapshots.ts';
import type { CompareMode, CompareSource } from '#lib/editor/compare.ts';
import { manifestToEdits } from '#lib/edits/manifest.ts';
import { originalPreviewEdits, type Edits } from '#lib/types/edits.ts';
import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
import { SingleFlight } from '#lib/utils/singleFlight.ts';

export interface CompareLeftCtx {
  assetId: string | null;
  edits: Edits;
  readonly compare: { readonly mode: CompareMode; readonly source: CompareSource };
  compareUrl: string | null;
  readonly snapshots: { readonly items: readonly EditSnapshot[] };
}

export interface CompareLeftHooks {
  proof(): ProofOptions;
  edge(): number;
}

export type CompareTarget = { assetId: string; edits: Edits | null; key: string };

type LeftArgs = CompareTarget & { edge: number };

export function compareTarget(ctx: CompareLeftCtx): CompareTarget | null {
  const assetId = ctx.assetId;
  if (!assetId) return null;
  const source = ctx.compare.source;
  if (source.kind === 'version')
    return { assetId: source.id, edits: null, key: `version:${source.id}` };
  if (source.kind === 'snapshot') {
    const snapshot = ctx.snapshots.items.find((item) => item.id === source.id);
    if (!snapshot) return null;
    return {
      assetId,
      edits: $state.snapshot(snapshot.edits) as Edits,
      key: `snapshot:${snapshot.id}:${snapshot.manifest_hash}`
    };
  }
  const edits = $state.snapshot(ctx.edits) as Edits;
  return {
    assetId,
    edits: originalPreviewEdits(edits),
    key: JSON.stringify({ g: edits.geometry, l: edits.lens, d: edits.color.dcp })
  };
}

export class CompareLeft {
  private readonly ctx: CompareLeftCtx;
  private readonly hooks: CompareLeftHooks;
  private edge = 0;
  private key = '';
  private saved: { id: string; edits: Edits } | null = null;

  constructor(ctx: CompareLeftCtx, hooks: CompareLeftHooks) {
    this.ctx = ctx;
    this.hooks = hooks;
  }

  private flight = new SingleFlight<LeftArgs, { url: string }>(
    async (args, signal) => {
      const edits = args.edits ?? (await this.savedEdits(args.assetId));
      signal.throwIfAborted();
      const { blob } = await livePreview(
        args.assetId,
        edits,
        args.edge,
        'none',
        this.hooks.proof(),
        signal,
        'original'
      );
      return { url: makeObjectUrl(blob) };
    },
    (args, result) => {
      const prev = this.ctx.compareUrl;
      this.ctx.compareUrl = result.url;
      if (prev?.startsWith('blob:')) revoke(prev);
      this.edge = args.edge;
      this.key = args.key;
    },
    () => {}
  );

  refresh(force = false): void {
    if (this.ctx.compare.mode === 'off') return;
    const target = compareTarget(this.ctx);
    if (!target) return;
    const edge = this.hooks.edge();
    if (!force && this.edge === edge && this.key === target.key && this.ctx.compareUrl) return;
    this.flight.submit({ ...target, edge });
  }

  drop(): void {
    this.flight.cancel();
    if (this.ctx.compareUrl?.startsWith('blob:')) revoke(this.ctx.compareUrl);
    this.ctx.compareUrl = null;
    this.edge = 0;
    this.key = '';
    this.saved = null;
  }

  private async savedEdits(assetId: string): Promise<Edits> {
    if (this.saved?.id === assetId) return this.saved.edits;
    const edits = manifestToEdits((await getEdits(assetId)).manifest);
    this.saved = { id: assetId, edits };
    return edits;
  }
}
