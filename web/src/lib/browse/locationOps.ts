import { assetSummaries, reverseGeocode, setAssetLocation, type PlaceName } from '#lib/api/map.ts';
import { withExif } from '#lib/browse/assetMeta.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { locationsMoved } from '#lib/map/locationMoves.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { places } from '#lib/stores/places.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo } from '#lib/types/asset.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface GeoLocation {
  lat: number;
  lon: number;
  place: PlaceName | null;
}

interface Previous {
  id: string;
  exif: ExifInfo | null;
}

export interface LocationPlan {
  owned: AssetSummary[];
  refused: number;
}

export function planLocation(assets: readonly AssetSummary[]): LocationPlan {
  const owned = assets.filter((asset) => ownsAsset(asset));
  return { owned, refused: assets.length - owned.length };
}

export function locationOf(exif: ExifInfo | null): GeoLocation | null {
  const lat = exif?.latitude;
  const lon = exif?.longitude;
  if (typeof lat !== 'number' || typeof lon !== 'number') return null;
  return {
    lat,
    lon,
    place: { city: exif?.city ?? null, state: exif?.state ?? null, country: exif?.country ?? null }
  };
}

function located(exif: ExifInfo | null, to: GeoLocation): ExifInfo {
  return withExif(exif, {
    latitude: to.lat,
    longitude: to.lon,
    city: to.place?.city ?? null,
    state: to.place?.state ?? null,
    country: to.place?.country ?? null
  });
}

function restore(previous: readonly Previous[]): void {
  for (const { id, exif } of previous) browsing.patch(id, { exifInfo: exif });
}

function uniqueSources(ids: readonly string[]): string[] {
  return [...new Set(ids.map(sourceId))];
}

async function write(ids: readonly string[], to: GeoLocation): Promise<void> {
  const sources = uniqueSources(ids);
  await setAssetLocation(sources, to.lat, to.lon);
  locationsMoved(sources.map((id) => ({ id, lat: to.lat, lon: to.lon, place: to.place })));
}

async function undo(previous: readonly Previous[], to: GeoLocation): Promise<void> {
  const back = previous.flatMap(({ id, exif }) => {
    const before = locationOf(exif);
    return before ? [{ id, before, exif }] : [];
  });
  const groups = new Map<string, { location: GeoLocation; ids: string[]; exif: Previous[] }>();
  for (const { id, before, exif } of back) {
    const key = `${before.lat},${before.lon}`;
    const group = groups.get(key) ?? { location: before, ids: [], exif: [] };
    group.ids.push(id);
    group.exif.push({ id, exif });
    groups.set(key, group);
  }
  try {
    for (const group of groups.values()) await write(group.ids, group.location);
  } catch (error) {
    toasts.fail('Undo location', error);
    return;
  }
  restore(back.map(({ id, exif }) => ({ id, exif })));
  libraryChanged();
  const kept = previous.length - back.length;
  if (kept > 0) {
    toasts.push(
      'warn',
      `${countOf(kept, 'photo')} had no location before and ${kept === 1 ? 'keeps' : 'keep'} the new one`,
      {
        description: `Immich cannot remove a location. It stays at ${to.lat.toFixed(4)}, ${to.lon.toFixed(4)}.`
      }
    );
  }
}

export async function applyLocation(
  assets: readonly AssetSummary[],
  to: GeoLocation
): Promise<boolean> {
  const { owned, refused } = planLocation(assets);
  if (owned.length === 0) {
    toasts.push('warn', 'Another Immich user owns these photos');
    return false;
  }
  if (!(await metadataConsent.gate())) return false;
  const previous: Previous[] = owned.map((asset) => ({ id: asset.id, exif: asset.exifInfo }));
  for (const asset of owned) browsing.patch(asset.id, { exifInfo: located(asset.exifInfo, to) });
  try {
    await write(
      owned.map((asset) => asset.id),
      to
    );
  } catch (error) {
    restore(previous);
    toasts.fail('Set location', error);
    return false;
  }
  libraryChanged();
  places.noteUse(to);
  toasts.push(
    'success',
    owned.length === 1 ? 'Location set' : `Location set for ${countOf(owned.length, 'photo')}`,
    {
      description:
        refused > 0 ? `${countOf(refused, 'photo')} owned by someone else kept theirs` : undefined,
      action: { label: 'Undo', run: () => void undo(previous, to) }
    }
  );
  return true;
}

export async function resolveAssets(ids: readonly string[]): Promise<AssetSummary[]> {
  const found = ids.map((id) => browsing.find(id));
  const missing = ids.filter((_, index) => !found[index]);
  const fetched = missing.length > 0 ? await assetSummaries(missing) : [];
  const byId = new Map(fetched.map((asset) => [asset.id, asset]));
  return ids.flatMap((id, index) => {
    const asset = found[index] ?? byId.get(sourceId(id));
    return asset ? [asset] : [];
  });
}

export async function placeAt(
  ids: readonly string[],
  lat: number,
  lon: number,
  named?: PlaceName | null
): Promise<boolean> {
  if (ids.length === 0) return false;
  let assets: AssetSummary[];
  try {
    assets = await resolveAssets(ids);
  } catch (error) {
    toasts.fail('Set location', error);
    return false;
  }
  const place = named !== undefined ? named : await reverseGeocode(lat, lon).catch(() => null);
  return applyLocation(assets, { lat, lon, place });
}
