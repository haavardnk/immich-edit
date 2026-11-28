import { sourceId } from '#lib/browse/assetKey.ts';
import { loadTagMembership } from '#lib/browse/tagMembership.ts';
import { membershipNeed, type TileInfo, type TileOverlay } from '#lib/browse/tileOverlay.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';

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

export function loadOverlayMembership(view: () => OverlayView): void {
  $effect(() => {
    const missing = missingMembership(view());
    if (missing.albums.length === 0 && missing.tags.length === 0) return;
    const timer = setTimeout(() => {
      if (missing.albums.length > 0) void assetAlbums.ensureMany(missing.albums);
      if (missing.tags.length > 0) void loadTagMembership(missing.tags);
    }, SETTLE_MS);
    return () => clearTimeout(timer);
  });
}
