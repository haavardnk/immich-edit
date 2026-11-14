import { updateAsset } from '#lib/api/assets.ts';
import { addTagToAsset, removeTagFromAsset } from '#lib/api/tags.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { rejected } from '#lib/stores/rejected.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { ensureRejectTag, isRejected, setRejectedTags } from '#lib/browse/reject.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import { slotQueue, type SlotQueue } from '#lib/browse/metaSlots.ts';
import {
  isLabelTag,
  labelOf,
  labelTagFor,
  withLabel,
  writeLabel,
  type LabelColor
} from '#lib/stores/labels.ts';
import { assignLabel } from '#lib/stores/labels.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { ExifInfo, TagRef } from '#lib/types/asset.ts';
import { errorMessage } from '#lib/utils/errors.ts';

export type AssetMeta = Pick<AssetSummary, 'ownerId' | 'isFavorite' | 'exifInfo' | 'tags'>;

export type MetaFields = Partial<Pick<AssetMeta, 'isFavorite' | 'exifInfo' | 'tags'>>;

export type MetaKind = 'rating' | 'favorite' | 'label' | 'reject' | 'tag' | 'description';

export type MetaOutcome = 'skipped' | 'saved' | 'failed';

export interface MetaTarget {
  readonly id: string;
  read(): AssetMeta | null;
  patch(fields: MetaFields): void;
  fail(kind: MetaKind, error: unknown): void;
}

export interface MetaChange<V> {
  slots: SlotQueue<V>;
  slot: string;
  from: V;
  to: V;
  fields: (asset: AssetMeta, value: V) => MetaFields;
  write: (saved: V) => Promise<unknown>;
  mirror?: (value: V) => void;
}

interface LabelValue {
  color: LabelColor | null;
  tags: TagRef[];
}

export const flagSlots = slotQueue<boolean>();
const ratingSlots = slotQueue<number | null>();
const descriptionSlots = slotQueue<string>();
const labelSlots = slotQueue<LabelValue>();

const TAG_UNAVAILABLE = 'could not create tag';

export function metaFailure(kind: MetaKind, error: unknown): string {
  return `${kind}: ${errorMessage(error)}`;
}

export function browseTarget(
  id: string,
  fail: MetaTarget['fail'] = (kind, error) => toasts.push('error', metaFailure(kind, error))
): MetaTarget {
  return {
    id,
    read: () => browsing.find(id),
    patch: (fields) => browsing.patch(id, fields),
    fail
  };
}

export async function writableMeta(target: MetaTarget): Promise<boolean> {
  const asset = target.read();
  if (!asset || !ownsAsset(asset)) return false;
  return metadataConsent.gate();
}

export async function changeMeta<V>(
  target: MetaTarget,
  kind: MetaKind,
  plan: (asset: AssetMeta) => MetaChange<V> | null
): Promise<MetaOutcome> {
  if (!(await writableMeta(target))) return 'skipped';
  return commit(target, kind, plan);
}

export function setRating(target: MetaTarget, rating: number | null): Promise<MetaOutcome> {
  return changeMeta(target, 'rating', (asset): MetaChange<number | null> => ({
    slots: ratingSlots,
    slot: 'rating',
    from: asset.exifInfo?.rating ?? null,
    to: rating,
    fields: (current, value) => ({ exifInfo: withExif(current.exifInfo, { rating: value }) }),
    write: () => updateAsset(target.id, { rating })
  }));
}

export function setDescription(target: MetaTarget, description: string): Promise<MetaOutcome> {
  return changeMeta(target, 'description', (asset): MetaChange<string> | null => {
    const from = asset.exifInfo?.description ?? '';
    if (from === description) return null;
    return {
      slots: descriptionSlots,
      slot: 'description',
      from,
      to: description,
      fields: (current, value) => ({
        exifInfo: withExif(current.exifInfo, { description: value })
      }),
      write: () => updateAsset(target.id, { description })
    };
  });
}

export function setFavorite(target: MetaTarget, value: boolean): Promise<MetaOutcome> {
  return changeMeta(target, 'favorite', (asset) => favoriteChange(target.id, asset, value));
}

export function toggleFavorite(target: MetaTarget): Promise<MetaOutcome> {
  return changeMeta(target, 'favorite', (asset) =>
    favoriteChange(target.id, asset, !asset.isFavorite)
  );
}

export function setRejected(target: MetaTarget, value: boolean): Promise<MetaOutcome> {
  return changeWithTag(target, 'reject', rejectTag, (asset, tag) =>
    rejectChange(target.id, asset, tag, value)
  );
}

export function toggleReject(target: MetaTarget): Promise<MetaOutcome> {
  return changeWithTag(target, 'reject', rejectTag, (asset, tag) =>
    rejectChange(target.id, asset, tag, !isRejected(asset))
  );
}

export function setLabel(target: MetaTarget, color: LabelColor | null): Promise<MetaOutcome> {
  return changeWithTag(
    target,
    'label',
    () => labelTagFor(color),
    (asset, tag): MetaChange<LabelValue> => ({
      slots: labelSlots,
      slot: 'label',
      from: { color: labelOf(asset), tags: asset.tags.filter(isLabelTag) },
      to: { color, tags: tag ? [tag] : [] },
      fields: (current, value) => ({ tags: withLabel(current.tags, value.tags[0] ?? null) }),
      write: (saved) => writeLabel(target.id, saved.tags, tag),
      mirror: (value) => assignLabel(target.id, value.color, value.tags[0] ?? null)
    })
  );
}

export async function clearFlags(target: MetaTarget): Promise<MetaOutcome> {
  const outcomes: MetaOutcome[] = [];
  if (target.read()?.isFavorite) outcomes.push(await setFavorite(target, false));
  const asset = target.read();
  if (asset && isRejected(asset)) outcomes.push(await setRejected(target, false));
  if (outcomes.includes('failed')) return 'failed';
  return outcomes.includes('saved') ? 'saved' : 'skipped';
}

export async function resolveTag<T extends TagRef | null>(
  kind: MetaKind,
  tagFor: () => Promise<T | undefined>,
  fail: MetaTarget['fail']
): Promise<T | undefined> {
  try {
    const tag = await tagFor();
    if (tag === undefined) fail(kind, TAG_UNAVAILABLE);
    return tag;
  } catch (error) {
    fail(kind, error);
    return undefined;
  }
}

export async function rejectTag(): Promise<TagRef | undefined> {
  return (await ensureRejectTag()) ?? undefined;
}

async function commit<V>(
  target: MetaTarget,
  kind: MetaKind,
  plan: (asset: AssetMeta) => MetaChange<V> | null
): Promise<MetaOutcome> {
  const asset = target.read();
  if (!asset) return 'skipped';
  const change = plan(asset);
  if (!change) return 'skipped';
  const result = await change.slots.write(`${target.id}:${change.slot}`, {
    from: change.from,
    to: change.to,
    show: (value) => {
      target.patch(change.fields(target.read() ?? asset, value));
      change.mirror?.(value);
    },
    write: change.write
  });
  if (result.ok) {
    libraryChanged();
    return 'saved';
  }
  target.fail(kind, result.error);
  return 'failed';
}

async function changeWithTag<T extends TagRef | null, V>(
  target: MetaTarget,
  kind: MetaKind,
  tagFor: () => Promise<T | undefined>,
  plan: (asset: AssetMeta, tag: T) => MetaChange<V>
): Promise<MetaOutcome> {
  if (!(await writableMeta(target))) return 'skipped';
  const tag = await resolveTag(kind, tagFor, target.fail);
  if (tag === undefined) return 'skipped';
  return commit(target, kind, (asset) => plan(asset, tag));
}

function favoriteChange(id: string, asset: AssetMeta, value: boolean): MetaChange<boolean> {
  return {
    slots: flagSlots,
    slot: 'favorite',
    from: asset.isFavorite,
    to: value,
    fields: (_, on) => ({ isFavorite: on }),
    write: () => updateAsset(id, { isFavorite: value })
  };
}

function rejectChange(
  id: string,
  asset: AssetMeta,
  tag: TagRef,
  value: boolean
): MetaChange<boolean> {
  return {
    slots: flagSlots,
    slot: 'reject',
    from: isRejected(asset),
    to: value,
    fields: (current, on) => ({ tags: setRejectedTags(current.tags, tag, on) }),
    write: () => (value ? addTagToAsset(tag.id, id) : removeTagFromAsset(tag.id, id)),
    mirror: (on) => markRejected(id, tag, on)
  };
}

function markRejected(id: string, tag: TagRef, value: boolean): void {
  if (value) rejected.add(id, tag);
  else rejected.remove(id);
}

function withExif(exif: ExifInfo | null, change: Partial<ExifInfo>): ExifInfo {
  if (exif) return { ...exif, ...change };
  return {
    make: null,
    model: null,
    lensModel: null,
    city: null,
    state: null,
    country: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal: null,
    timeZone: null,
    rating: null,
    fileSizeInByte: null,
    description: null,
    latitude: null,
    longitude: null,
    ...change
  };
}
