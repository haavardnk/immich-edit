import { addTagToAsset, removeTagFromAsset, upsertTags } from '#lib/api/tags.ts';
import {
  changeMeta,
  flagSlots,
  writableMeta,
  type MetaChange,
  type MetaOutcome,
  type MetaTarget
} from '#lib/browse/assetMeta.ts';
import { toTagRef } from '#lib/browse/managedTags.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';

export function addTag(target: MetaTarget, tag: TagRef): Promise<MetaOutcome> {
  return changeMeta(target, 'tag', (asset) => tagChange(target.id, asset.tags, tag.id, tag, true));
}

export function removeTag(target: MetaTarget, tagId: string): Promise<MetaOutcome> {
  return changeMeta(target, 'tag', (asset) => {
    const applied = asset.tags.find((t) => t.id === tagId) ?? null;
    return tagChange(target.id, asset.tags, tagId, applied, false);
  });
}

export async function createAndAddTag(target: MetaTarget, value: string): Promise<TagRef | null> {
  if (!(await writableMeta(target))) return null;
  let tag: TagRef | null;
  try {
    tag = await createTag(value);
  } catch (error) {
    target.fail('tag', error);
    return null;
  }
  if (!tag) return null;
  return (await addTag(target, tag)) === 'saved' ? tag : null;
}

async function createTag(value: string): Promise<TagRef | null> {
  const [created] = await upsertTags([value]);
  return created ? toTagRef(created) : null;
}

function tagChange(
  assetId: string,
  tags: TagRef[],
  tagId: string,
  tag: TagRef | null,
  on: boolean
): MetaChange<boolean> {
  return {
    slots: flagSlots,
    slot: `tag:${tagId}`,
    from: tags.some((t) => t.id === tagId),
    to: on,
    fields: (asset, value) => ({ tags: withTag(asset.tags, tagId, value ? tag : null) }),
    write: () => (on ? addTagToAsset(tagId, assetId) : removeTagFromAsset(tagId, assetId)),
    mirror: (value) =>
      tagCache.update(assetId, (cached) => withTag(cached, tagId, value ? tag : null))
  };
}

function withTag(tags: TagRef[], tagId: string, tag: TagRef | null): TagRef[] {
  const applied = tags.some((t) => t.id === tagId);
  if (tag) return applied ? tags : [...tags, tag];
  return applied ? tags.filter((t) => t.id !== tagId) : tags;
}
