import { updateAsset } from '$lib/api/assets';
import { addTagToAsset, removeTagFromAsset, upsertTags } from '$lib/api/tags';
import { browsing } from '$lib/stores/browsing.svelte';
import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
import { rejected } from '$lib/stores/rejected.svelte';
import { ensureRejectTag, isRejected, setRejectedTags } from '$lib/reject';
import type { AssetDetail, ExifInfo, TagRef } from '$lib/types/asset';
import { errorMessage } from '$lib/utils/errors';

export interface MetadataCtx {
  assetId: string | null;
  asset: AssetDetail | null;
  error: string | null;
}

type Loaded = MetadataCtx & { assetId: string; asset: AssetDetail };

async function ready(ctx: MetadataCtx): Promise<Loaded | null> {
  if (!ctx.assetId || !ctx.asset) return null;
  if (!(await metadataConsent.gate())) return null;
  return ctx as Loaded;
}

function syncBrowsing(ctx: Loaded): void {
  browsing.patch(ctx.assetId, {
    isFavorite: ctx.asset.isFavorite,
    exifInfo: ctx.asset.exifInfo ?? null,
    tags: ctx.asset.tags
  });
}

function blankExif(rating: number | null): ExifInfo {
  return {
    make: null,
    model: null,
    lensModel: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal: null,
    rating,
    fileSizeInByte: null
  };
}

export async function toggleFavorite(ctx: MetadataCtx): Promise<void> {
  const c = await ready(ctx);
  if (!c) return;
  const prev = c.asset.isFavorite;
  c.asset = { ...c.asset, isFavorite: !prev };
  try {
    const updated = await updateAsset(c.assetId, { isFavorite: !prev });
    c.asset = { ...updated, tags: c.asset.tags };
    syncBrowsing(c);
  } catch (e) {
    if (c.asset) c.asset = { ...c.asset, isFavorite: prev };
    c.error = errorMessage(e);
  }
}

export async function setRating(ctx: MetadataCtx, rating: number | null): Promise<void> {
  const c = await ready(ctx);
  if (!c) return;
  const prevExif = c.asset.exifInfo;
  c.asset = {
    ...c.asset,
    exifInfo: prevExif ? { ...prevExif, rating } : blankExif(rating)
  };
  try {
    const updated = await updateAsset(c.assetId, { rating });
    c.asset = { ...updated, tags: c.asset.tags };
    syncBrowsing(c);
  } catch (e) {
    if (c.asset) c.asset = { ...c.asset, exifInfo: prevExif };
    c.error = errorMessage(e);
  }
}

export async function addTag(ctx: MetadataCtx, tag: TagRef): Promise<void> {
  if (ctx.asset?.tags.some((t) => t.id === tag.id)) return;
  const c = await ready(ctx);
  if (!c) return;
  const prev = c.asset.tags;
  c.asset = { ...c.asset, tags: [...prev, tag] };
  try {
    await addTagToAsset(tag.id, c.assetId);
    syncBrowsing(c);
  } catch (e) {
    if (c.asset) c.asset = { ...c.asset, tags: prev };
    c.error = errorMessage(e);
  }
}

export async function removeTag(ctx: MetadataCtx, tagId: string): Promise<void> {
  const c = await ready(ctx);
  if (!c) return;
  const prev = c.asset.tags;
  c.asset = { ...c.asset, tags: prev.filter((t) => t.id !== tagId) };
  try {
    await removeTagFromAsset(tagId, c.assetId);
    syncBrowsing(c);
  } catch (e) {
    if (c.asset) c.asset = { ...c.asset, tags: prev };
    c.error = errorMessage(e);
  }
}

export async function toggleReject(ctx: MetadataCtx): Promise<void> {
  const c = await ready(ctx);
  if (!c) return;
  const rejectTag = await ensureRejectTag();
  if (!rejectTag) {
    c.error = 'reject: could not create tag';
    return;
  }
  const next = !isRejected(c.asset);
  const prev = c.asset.tags;
  c.asset = { ...c.asset, tags: setRejectedTags(prev, rejectTag, next) };
  if (next) rejected.add(c.assetId, rejectTag);
  else rejected.remove(c.assetId);
  try {
    if (next) await addTagToAsset(rejectTag.id, c.assetId);
    else await removeTagFromAsset(rejectTag.id, c.assetId);
    syncBrowsing(c);
  } catch (e) {
    if (c.asset) c.asset = { ...c.asset, tags: prev };
    if (next) rejected.remove(c.assetId);
    else rejected.add(c.assetId, rejectTag);
    c.error = errorMessage(e);
  }
}

export async function createAndAddTag(ctx: MetadataCtx, value: string): Promise<TagRef | null> {
  const c = await ready(ctx);
  if (!c) return null;
  try {
    const created = await upsertTags([value]);
    const tag = created[0];
    if (!tag) return null;
    const ref: TagRef = { id: tag.id, name: tag.name, value: tag.value };
    if (c.asset.tags.some((t) => t.id === ref.id)) return ref;
    const prev = c.asset.tags;
    c.asset = { ...c.asset, tags: [...prev, ref] };
    try {
      await addTagToAsset(ref.id, c.assetId);
      syncBrowsing(c);
    } catch (e) {
      if (c.asset) c.asset = { ...c.asset, tags: prev };
      c.error = errorMessage(e);
      return null;
    }
    return ref;
  } catch (e) {
    c.error = errorMessage(e);
    return null;
  }
}
