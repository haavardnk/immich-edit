import * as assetMeta from '#lib/browse/assetMeta.ts';
import * as tagOps from '#lib/browse/tagOps.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import type { LabelColor } from '#lib/stores/labels.ts';
import type { AssetDetail, TagRef } from '#lib/types/asset.ts';

export interface MetadataCtx {
  assetId: string | null;
  asset: AssetDetail | null;
  error: string | null;
}

function editorTarget(ctx: MetadataCtx): assetMeta.MetaTarget | null {
  const id = ctx.assetId;
  if (!id) return null;
  const open = (): boolean => ctx.assetId === id;
  return {
    id,
    read: () => (open() ? ctx.asset : null),
    patch: (fields) => {
      if (open() && ctx.asset) ctx.asset = { ...ctx.asset, ...fields };
      browsing.patch(id, fields);
    },
    fail: (kind, error) => {
      if (open()) ctx.error = assetMeta.metaFailure(kind, error);
    }
  };
}

async function run(
  ctx: MetadataCtx,
  op: (target: assetMeta.MetaTarget) => Promise<assetMeta.MetaOutcome>
): Promise<void> {
  const target = editorTarget(ctx);
  if (target) await op(target);
}

export function toggleFavorite(ctx: MetadataCtx): Promise<void> {
  return run(ctx, assetMeta.toggleFavorite);
}

export function setRating(ctx: MetadataCtx, rating: number | null): Promise<void> {
  return run(ctx, (target) => assetMeta.setRating(target, rating));
}

export function toggleReject(ctx: MetadataCtx): Promise<void> {
  return run(ctx, assetMeta.toggleReject);
}

export function setLabel(ctx: MetadataCtx, color: LabelColor | null): Promise<void> {
  return run(ctx, (target) => assetMeta.setLabel(target, color));
}

export function clearFlags(ctx: MetadataCtx): Promise<void> {
  return run(ctx, assetMeta.clearFlags);
}

export function addTag(ctx: MetadataCtx, tag: TagRef): Promise<void> {
  return run(ctx, (target) => tagOps.addTag(target, tag));
}

export function removeTag(ctx: MetadataCtx, tagId: string): Promise<void> {
  return run(ctx, (target) => tagOps.removeTag(target, tagId));
}

export function createAndAddTag(ctx: MetadataCtx, value: string): Promise<TagRef | null> {
  const target = editorTarget(ctx);
  return target ? tagOps.createAndAddTag(target, value) : Promise.resolve(null);
}
