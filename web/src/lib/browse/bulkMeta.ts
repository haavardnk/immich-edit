import * as assetMeta from '#lib/browse/assetMeta.ts';
import * as tagOps from '#lib/browse/tagOps.ts';
import { LABEL_NAMES, labelTagFor, type LabelColor } from '#lib/stores/labels.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { countOf } from '#lib/utils/plural.ts';
import { countFailures } from '#lib/utils/pool.ts';

type Op = (target: assetMeta.MetaTarget) => Promise<assetMeta.MetaOutcome>;

const CONCURRENCY = 6;

export function setRating(ids: string[], rating: number | null): Promise<boolean> {
  return apply(ids, (target) => assetMeta.setRating(target, rating), updated);
}

export function setFavorite(ids: string[], value: boolean): Promise<boolean> {
  return apply(ids, (target) => assetMeta.setFavorite(target, value), updated);
}

export function setRejected(ids: string[], value: boolean): Promise<boolean> {
  return apply(
    ids,
    (target) => assetMeta.setRejected(target, value),
    (count) => `${value ? 'Rejected' : 'Unrejected'} ${countOf(count, 'asset')}`,
    () => tagReady('reject', assetMeta.rejectTag)
  );
}

export function setLabel(ids: string[], color: LabelColor | null): Promise<boolean> {
  const what = color ? `${LABEL_NAMES[color]} label on` : 'Cleared the label of';
  return apply(
    ids,
    (target) => assetMeta.setLabel(target, color),
    (count) => `${what} ${countOf(count, 'photo')}`,
    () => tagReady('label', () => labelTagFor(color))
  );
}

export function addTags(ids: string[], tags: TagRef[]): Promise<boolean> {
  return applyTags(ids, tags, true);
}

export function removeTags(ids: string[], tags: TagRef[]): Promise<boolean> {
  return applyTags(ids, tags, false);
}

async function apply(
  ids: string[],
  op: Op,
  success: (count: number) => string,
  ready: () => Promise<boolean> = () => Promise.resolve(true)
): Promise<boolean> {
  if (!(await metadataConsent.gate()) || !(await ready())) return false;
  const failed = await countFailures(ids, CONCURRENCY, (id) => saved(op(silentTarget(id))));
  toasts.tally(failed, `${failed} of ${ids.length} failed`, success(ids.length));
  return true;
}

async function applyTags(ids: string[], tags: TagRef[], add: boolean): Promise<boolean> {
  if (!(await metadataConsent.gate())) return false;
  const pairs = ids.flatMap((id) => tags.map((tag) => ({ id, tag })));
  const failed = await countFailures(pairs, CONCURRENCY, ({ id, tag }) =>
    saved(add ? tagOps.addTag(silentTarget(id), tag) : tagOps.removeTag(silentTarget(id), tag.id))
  );
  toasts.tally(
    failed,
    `${countOf(failed, 'tag update')} failed`,
    `${add ? 'Added' : 'Removed'} tags on ${countOf(ids.length, 'asset')}`
  );
  return true;
}

async function saved(outcome: Promise<assetMeta.MetaOutcome>): Promise<boolean> {
  return (await outcome) === 'saved';
}

async function tagReady(
  kind: assetMeta.MetaKind,
  tagFor: () => Promise<TagRef | null | undefined>
): Promise<boolean> {
  const tag = await assetMeta.resolveTag(kind, tagFor, (failedKind, error) =>
    toasts.push('error', assetMeta.metaFailure(failedKind, error), { ttlMs: 6000 })
  );
  return tag !== undefined;
}

function silentTarget(id: string): assetMeta.MetaTarget {
  return assetMeta.browseTarget(id, () => {});
}

function updated(count: number): string {
  return `Updated ${countOf(count, 'asset')}`;
}
