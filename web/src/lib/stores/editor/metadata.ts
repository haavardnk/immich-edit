import { addTagToAsset, removeTagFromAsset, upsertTags } from '#lib/api/tags.ts';
import * as assetMeta from '#lib/browse/assetMeta.ts';
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

function addTagTo(target: assetMeta.MetaTarget, tag: TagRef): Promise<assetMeta.MetaOutcome> {
  return assetMeta.changeMeta(target, 'tag', (asset) =>
    hasTag(asset, tag)
      ? null
      : { fields: { tags: [...asset.tags, tag] }, write: () => addTagToAsset(tag.id, target.id) }
  );
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
  if (hasTag(ctx.asset, tag)) return Promise.resolve();
  return run(ctx, (target) => addTagTo(target, tag));
}

export function removeTag(ctx: MetadataCtx, tagId: string): Promise<void> {
  return run(ctx, (target) =>
    assetMeta.changeMeta(target, 'tag', (asset) => ({
      fields: { tags: asset.tags.filter((t) => t.id !== tagId) },
      write: () => removeTagFromAsset(tagId, target.id)
    }))
  );
}

export async function createAndAddTag(ctx: MetadataCtx, value: string): Promise<TagRef | null> {
  const target = editorTarget(ctx);
  if (!target || !(await assetMeta.writableMeta(target))) return null;
  let tag: TagRef | null;
  try {
    tag = await createTag(value);
  } catch (error) {
    target.fail('tag', error);
    return null;
  }
  if (!tag) return null;
  if ((await addTagTo(target, tag)) === 'saved' || hasTag(target.read(), tag)) return tag;
  return null;
}

function hasTag(asset: { tags: TagRef[] } | null, tag: TagRef): boolean {
  return asset?.tags.some((t) => t.id === tag.id) ?? false;
}

async function createTag(value: string): Promise<TagRef | null> {
  const [created] = await upsertTags([value]);
  return created ? { id: created.id, name: created.name, value: created.value } : null;
}
