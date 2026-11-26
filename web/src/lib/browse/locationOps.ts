import {
  reverseGeocode,
  setAssetLocation,
  setAssetLocations,
  type PlaceName
} from '#lib/api/map.ts';
import { withExif } from '#lib/browse/assetMeta.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import { resolveAssets } from '#lib/browse/assetLookup.ts';
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

export interface Placement {
  id: string;
  to: GeoLocation;
}

async function writeEach(placements: readonly Placement[]): Promise<Set<string>> {
  const bySource = new Map(placements.map((placement) => [sourceId(placement.id), placement]));
  const { failed } = await setAssetLocations(
    [...bySource].map(([id, { to }]) => ({ id, latitude: to.lat, longitude: to.lon }))
  );
  const refused = new Set(failed);
  locationsMoved(
    [...bySource]
      .filter(([id]) => !refused.has(id))
      .map(([id, { to }]) => ({ id, lat: to.lat, lon: to.lon, place: to.place }))
  );
  return refused;
}

async function undo(previous: readonly Previous[]): Promise<void> {
  const back = previous.flatMap(({ id, exif }) => {
    const before = locationOf(exif);
    return before ? [{ id, to: before, exif }] : [];
  });
  try {
    if (back.length > 0) await writeEach(back);
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
      { description: 'Immich cannot remove a location.' }
    );
  }
}

async function ownedWithConsent(assets: readonly AssetSummary[]): Promise<LocationPlan | null> {
  const plan = planLocation(assets);
  if (plan.owned.length === 0) {
    toasts.push('warn', 'Another Immich user owns these photos');
    return null;
  }
  return (await metadataConsent.gate()) ? plan : null;
}

function refusedLine(refused: number): string | undefined {
  return refused > 0 ? `${countOf(refused, 'photo')} owned by someone else kept theirs` : undefined;
}

export interface FollowUp {
  note: string;
  undo: () => Promise<void>;
}

export async function applyLocations(
  assets: readonly AssetSummary[],
  targets: ReadonlyMap<string, GeoLocation>,
  then?: (written: readonly string[]) => Promise<FollowUp | null>
): Promise<boolean> {
  const plan = await ownedWithConsent(assets.filter((asset) => targets.has(asset.id)));
  if (!plan) return false;
  const previous: Previous[] = plan.owned.map((asset) => ({ id: asset.id, exif: asset.exifInfo }));
  const placements = plan.owned.flatMap((asset) => {
    const to = targets.get(asset.id);
    return to ? [{ id: asset.id, to }] : [];
  });
  for (const { id, to } of placements) {
    const asset = plan.owned.find((owned) => owned.id === id);
    browsing.patch(id, { exifInfo: located(asset?.exifInfo ?? null, to) });
  }
  let failed: Set<string>;
  try {
    failed = await writeEach(placements);
  } catch (error) {
    restore(previous);
    toasts.fail('Set locations', error);
    return false;
  }
  restore(previous.filter(({ id }) => failed.has(sourceId(id))));
  const written = previous.filter(({ id }) => !failed.has(sourceId(id)));
  libraryChanged();
  const refused = plan.refused + previous.length - written.length;
  const extra = then && written.length > 0 ? await then(written.map(({ id }) => id)) : null;
  const title = `Placed ${countOf(written.length, 'photo')}`;
  toasts.push('success', extra ? `${title} ${extra.note}` : title, {
    description: refusedLine(refused),
    action: {
      label: 'Undo',
      run: () => {
        void undo(written);
        void extra?.undo();
      }
    }
  });
  return written.length > 0;
}

export async function applyLocation(
  assets: readonly AssetSummary[],
  to: GeoLocation
): Promise<boolean> {
  const plan = await ownedWithConsent(assets);
  if (!plan) return false;
  const { owned, refused } = plan;
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
      description: refusedLine(refused),
      action: { label: 'Undo', run: () => void undo(previous) }
    }
  );
  return true;
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
