import { resolveAssets } from '#lib/browse/assetLookup.ts';
import { applyLocation, locationOf, type GeoLocation } from '#lib/browse/locationOps.ts';
import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export const NO_LOCATION_COPIED = 'Copy a location first';

export const NO_LOCATION_TO_COPY = 'This photo has no location';

export function locationName(location: GeoLocation): string {
  const place = location.place;
  return (
    place?.city ?? place?.state ?? place?.country ?? coordinateLabel(location.lat, location.lon)
  );
}

export function copyLocationOf(asset: AssetSummary): void {
  const location = locationOf(asset.exifInfo);
  if (!location) {
    toasts.push('info', NO_LOCATION_TO_COPY);
    return;
  }
  locationClipboard.copy(location);
  toasts.push('success', `Copied ${locationName(location)}`);
}

export async function pasteLocationOnto(assets: readonly AssetSummary[]): Promise<boolean> {
  const copied = locationClipboard.copied;
  if (!copied) {
    toasts.push('info', NO_LOCATION_COPIED);
    return false;
  }
  return applyLocation(assets, copied);
}

async function resolved(ids: readonly string[]): Promise<AssetSummary[] | null> {
  try {
    return await resolveAssets(ids);
  } catch (error) {
    toasts.fail('Location', error);
    return null;
  }
}

export async function copyLocationFrom(id: string): Promise<void> {
  const asset = (await resolved([id]))?.[0];
  if (asset) copyLocationOf(asset);
}

export async function pasteLocationTo(ids: readonly string[]): Promise<boolean> {
  if (ids.length === 0) return false;
  if (!locationClipboard.copied) return pasteLocationOnto([]);
  const assets = await resolved(ids);
  return assets ? pasteLocationOnto(assets) : false;
}
