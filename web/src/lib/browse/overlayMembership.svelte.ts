import { tagMemberships } from '#lib/api/tags.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { toTagRef } from '#lib/browse/managedTags.ts';
import { membershipNeed, type TileInfo, type TileOverlay } from '#lib/browse/tileOverlay.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';

export const SETTLE_MS = 150;
export const MAX_VISIBLE = 500;

export interface OverlayView {
  info: TileInfo;
  overlay: TileOverlay;
  ids: readonly string[];
}

export interface MissingMembership {
  albums: string[];
  tags: string[];
}

export function missingMembership(view: OverlayView): MissingMembership {
  const need = membershipNeed(view.info, view.overlay);
  if (!need.albums && !need.tags) return { albums: [], tags: [] };
  const sources = [...new Set(view.ids.map(sourceId))].slice(0, MAX_VISIBLE);
  return {
    albums: need.albums ? sources.filter((id) => !assetAlbums.has(id)) : [],
    tags: need.tags ? sources.filter((id) => tagCache.get(id) === undefined) : []
  };
}

async function fetchTags(ids: readonly string[]): Promise<Record<string, TagRef[]>> {
  const found = await tagMemberships(ids);
  return Object.fromEntries(Object.entries(found).map(([id, tags]) => [id, tags.map(toTagRef)]));
}

export function loadOverlayMembership(view: () => OverlayView): void {
  $effect(() => {
    const missing = missingMembership(view());
    if (missing.albums.length === 0 && missing.tags.length === 0) return;
    const timer = setTimeout(() => {
      if (missing.albums.length > 0) void assetAlbums.ensureMany(missing.albums);
      if (missing.tags.length > 0) void tagCache.loadMany(missing.tags, fetchTags);
    }, SETTLE_MS);
    return () => clearTimeout(timer);
  });
}
