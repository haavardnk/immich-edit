import { getContext, setContext } from 'svelte';
import { browseTarget, type MetaTarget } from '#lib/browse/assetMeta.ts';
import { loupeTags } from '#lib/stores/loupeTags.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';

export type InfoSource = {
  target: (id: string) => MetaTarget | null;
  tagsOf: (id: string) => TagRef[];
  addTag: (id: string, tag: TagRef) => unknown;
  removeTag: (id: string, tagId: string) => unknown;
  createTag: (id: string, value: string) => Promise<TagRef | null>;
};

const KEY = Symbol('infoSource');

const BROWSE_INFO: InfoSource = {
  target: (id) => browseTarget(id),
  tagsOf: (id) => loupeTags.tagsOf(id),
  addTag: (id, tag) => loupeTags.add(id, tag),
  removeTag: (id, tagId) => loupeTags.remove(id, tagId),
  createTag: (id, value) => loupeTags.createAndAdd(id, value)
};

export function provideInfoSource(source: InfoSource): void {
  setContext(KEY, source);
}

export function infoSource(): InfoSource {
  return getContext<InfoSource | undefined>(KEY) ?? BROWSE_INFO;
}
