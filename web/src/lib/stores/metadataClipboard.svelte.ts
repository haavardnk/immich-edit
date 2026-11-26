import { getAsset } from '#lib/api/assets.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { addTags, setRating } from '#lib/browse/bulkMeta.ts';
import { resolveAssets } from '#lib/browse/assetLookup.ts';
import { applyLocation, locationOf, type GeoLocation } from '#lib/browse/locationOps.ts';
import { isManagedTag } from '#lib/browse/managedTags.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import { setDescriptions } from '#lib/metadata/descriptionOps.ts';
import { descriptionChanges, descriptionOf } from '#lib/metadata/descriptions.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { storedLocation } from '#lib/stores/locationClipboard.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { isRecord } from '#lib/utils/isRecord.ts';
import { countOf } from '#lib/utils/plural.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

export type MetadataField = 'description' | 'location' | 'tags' | 'rating';

export interface CopiedMetadata {
  name: string;
  description: string;
  location: GeoLocation | null;
  tags: TagRef[];
  rating: number | null;
}

export const METADATA_FIELDS: readonly MetadataField[] = [
  'description',
  'location',
  'tags',
  'rating'
];

const COPIED_KEY = 'immich-edit:copiedMetadata';
const FIELDS_KEY = 'immich-edit:pasteFields';
const DEFAULT_FIELDS: readonly MetadataField[] = ['description', 'location', 'tags'];

function isField(value: unknown): value is MetadataField {
  return METADATA_FIELDS.includes(value as MetadataField);
}

function isTag(value: unknown): value is TagRef {
  return (
    isRecord(value) &&
    typeof value.id === 'string' &&
    typeof value.name === 'string' &&
    typeof value.value === 'string'
  );
}

function ratingOf(value: unknown): number | null {
  return typeof value === 'number' && Number.isInteger(value) && value >= 1 && value <= 5
    ? value
    : null;
}

export function storedMetadata(raw: unknown): CopiedMetadata | null {
  if (!isRecord(raw) || typeof raw.name !== 'string') return null;
  return {
    name: raw.name,
    description: typeof raw.description === 'string' ? raw.description : '',
    location: storedLocation(raw.location),
    tags: Array.isArray(raw.tags) ? raw.tags.filter(isTag) : [],
    rating: ratingOf(raw.rating)
  };
}

function storedFields(raw: unknown): MetadataField[] {
  return Array.isArray(raw) ? raw.filter(isField) : [...DEFAULT_FIELDS];
}

class MetadataClipboard {
  copied = $state.raw<CopiedMetadata | null>(storedMetadata(readStored(COPIED_KEY, 'session')));
  fields = $state<MetadataField[]>(storedFields(readStored<unknown[]>(FIELDS_KEY)));
  pasting = $state.raw<AssetSummary[] | null>(null);
  busy = $state(false);

  copy = async (asset: AssetSummary): Promise<void> => {
    let tags: TagRef[] = [];
    try {
      if (ownsAsset(asset)) {
        tags = (await getAsset(asset.id)).tags.filter((tag) => !isManagedTag(tag));
      }
    } catch (error) {
      toasts.fail('Copy metadata', error);
      return;
    }
    const location = locationOf(asset.exifInfo);
    const copied: CopiedMetadata = {
      name: asset.originalFileName,
      description: descriptionOf(asset),
      location,
      tags,
      rating: ratingOf(asset.exifInfo?.rating)
    };
    this.copied = copied;
    writeStored(COPIED_KEY, copied, 'session');
    toasts.push('success', `Copied the metadata of ${asset.originalFileName}`);
  };

  toggle = (field: MetadataField, on: boolean): void => {
    this.fields = on
      ? METADATA_FIELDS.filter((f) => f === field || this.fields.includes(f))
      : this.fields.filter((f) => f !== field);
    writeStored(FIELDS_KEY, this.fields);
  };

  openPaste = async (ids: readonly string[]): Promise<void> => {
    try {
      const assets = await resolveAssets(ids);
      const sources = new Set<string>();
      const owned = assets.filter((asset) => {
        const source = sourceId(asset.id);
        if (!ownsAsset(asset) || sources.has(source)) return false;
        sources.add(source);
        return true;
      });
      if (owned.length === 0) {
        toasts.push('warn', 'Another Immich user owns these photos');
        return;
      }
      const others = new Set(
        assets.filter((asset) => !ownsAsset(asset)).map((a) => sourceId(a.id))
      );
      if (others.size > 0) {
        toasts.push('info', `${countOf(others.size, 'photo')} owned by someone else left out`);
      }
      this.pasting = owned;
    } catch (error) {
      toasts.fail('Paste metadata', error);
    }
  };

  close = (): void => {
    if (!this.busy) this.pasting = null;
  };

  paste = async (): Promise<void> => {
    const copied = this.copied;
    const assets = this.pasting;
    if (!copied || !assets || this.busy) return;
    const ids = assets.map((asset) => asset.id);
    const on = new Set(this.fields);
    if (!(await metadataConsent.gate())) return;
    this.busy = true;
    try {
      if (on.has('description') && copied.description) {
        await setDescriptions(descriptionChanges(assets, copied.description));
      }
      if (on.has('location') && copied.location) await applyLocation(assets, copied.location);
      if (on.has('tags') && copied.tags.length > 0) await addTags(ids, copied.tags);
      if (on.has('rating') && copied.rating) await setRating(ids, copied.rating);
      this.pasting = null;
    } finally {
      this.busy = false;
    }
  };
}

export const metadataClipboard = new MetadataClipboard();
