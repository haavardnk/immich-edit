import { updateAsset } from '#lib/api/assets.ts';
import { addTagToAsset, removeTagFromAsset } from '#lib/api/tags.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { rejected } from '#lib/stores/rejected.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { ensureRejectTag, isRejected, setRejectedTags } from '#lib/browse/reject.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
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

type AssetMeta = Pick<AssetSummary, 'ownerId' | 'isFavorite' | 'exifInfo' | 'tags'>;

type MetaFields = Partial<Pick<AssetMeta, 'isFavorite' | 'exifInfo' | 'tags'>>;

export type MetaKind = 'rating' | 'favorite' | 'label' | 'reject' | 'tag';

export type MetaOutcome = 'skipped' | 'saved' | 'failed';

export interface MetaTarget {
  readonly id: string;
  read(): AssetMeta | null;
  patch(fields: MetaFields): void;
  fail(kind: MetaKind, error: unknown): void;
}

interface MetaChange {
  fields: MetaFields;
  write: () => Promise<unknown>;
  apply?: () => void;
  revert?: () => void;
}

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

export async function changeMeta(
  target: MetaTarget,
  kind: MetaKind,
  plan: (asset: AssetMeta) => MetaChange | null
): Promise<MetaOutcome> {
  if (!(await writableMeta(target))) return 'skipped';
  return commit(target, kind, plan);
}

export function setRating(target: MetaTarget, rating: number | null): Promise<MetaOutcome> {
  return changeMeta(target, 'rating', (asset) => ({
    fields: { exifInfo: withRating(asset.exifInfo, rating) },
    write: () => updateAsset(target.id, { rating })
  }));
}

export function setFavorite(target: MetaTarget, value: boolean): Promise<MetaOutcome> {
  return changeMeta(target, 'favorite', () => favoriteChange(target.id, value));
}

export function toggleFavorite(target: MetaTarget): Promise<MetaOutcome> {
  return changeMeta(target, 'favorite', (asset) => favoriteChange(target.id, !asset.isFavorite));
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
    (asset, tag) => {
      const tags = asset.tags;
      const previousColor = labelOf(asset);
      const previousTag = tags.find(isLabelTag) ?? null;
      return {
        fields: { tags: withLabel(tags, tag) },
        write: () => writeLabel(target.id, tags, tag),
        apply: () => assignLabel(target.id, color, tag),
        revert: () => assignLabel(target.id, previousColor, previousTag)
      };
    }
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

async function commit(
  target: MetaTarget,
  kind: MetaKind,
  plan: (asset: AssetMeta) => MetaChange | null
): Promise<MetaOutcome> {
  const asset = target.read();
  if (!asset) return 'skipped';
  const change = plan(asset);
  if (!change) return 'skipped';
  const previous = previousOf(asset, change.fields);
  target.patch(change.fields);
  change.apply?.();
  try {
    await change.write();
    return 'saved';
  } catch (error) {
    target.patch(previous);
    change.revert?.();
    target.fail(kind, error);
    return 'failed';
  }
}

async function changeWithTag<T extends TagRef | null>(
  target: MetaTarget,
  kind: MetaKind,
  tagFor: () => Promise<T | undefined>,
  plan: (asset: AssetMeta, tag: T) => MetaChange
): Promise<MetaOutcome> {
  if (!(await writableMeta(target))) return 'skipped';
  const tag = await resolveTag(target, kind, tagFor);
  if (tag === undefined) return 'skipped';
  return commit(target, kind, (asset) => plan(asset, tag));
}

async function resolveTag<T extends TagRef | null>(
  target: MetaTarget,
  kind: MetaKind,
  tagFor: () => Promise<T | undefined>
): Promise<T | undefined> {
  try {
    const tag = await tagFor();
    if (tag === undefined) target.fail(kind, TAG_UNAVAILABLE);
    return tag;
  } catch (error) {
    target.fail(kind, error);
    return undefined;
  }
}

function previousOf(asset: AssetMeta, fields: MetaFields): MetaFields {
  const previous: MetaFields = {};
  if ('isFavorite' in fields) previous.isFavorite = asset.isFavorite;
  if ('exifInfo' in fields) previous.exifInfo = asset.exifInfo;
  if ('tags' in fields) previous.tags = asset.tags;
  return previous;
}

function favoriteChange(id: string, value: boolean): MetaChange {
  return { fields: { isFavorite: value }, write: () => updateAsset(id, { isFavorite: value }) };
}

async function rejectTag(): Promise<TagRef | undefined> {
  return (await ensureRejectTag()) ?? undefined;
}

function rejectChange(id: string, asset: AssetMeta, tag: TagRef, value: boolean): MetaChange {
  return {
    fields: { tags: setRejectedTags(asset.tags, tag, value) },
    write: () => (value ? addTagToAsset(tag.id, id) : removeTagFromAsset(tag.id, id)),
    apply: () => markRejected(id, tag, value),
    revert: () => markRejected(id, tag, !value)
  };
}

function markRejected(id: string, tag: TagRef, value: boolean): void {
  if (value) rejected.add(id, tag);
  else rejected.remove(id);
}

function withRating(exif: ExifInfo | null, rating: number | null): ExifInfo {
  if (exif) return { ...exif, rating };
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
    rating,
    fileSizeInByte: null
  };
}
