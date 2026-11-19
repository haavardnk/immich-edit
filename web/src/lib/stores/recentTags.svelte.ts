import type { TagSummary } from '#lib/api/tags.ts';
import {
  liveRecentTags,
  sameRecentTags,
  storedRecentTags,
  withRecentTag
} from '#lib/browse/recentTags.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:recentTags';

type Persisted = { tags: TagRef[] };

class RecentTagsStore {
  tags = $state<TagRef[]>(storedRecentTags(readStored<Persisted>(STORAGE_KEY)?.tags));

  record = (tag: TagRef): void => {
    this.#set(withRecentTag(this.tags, tag));
  };

  keep = (live: readonly TagSummary[]): void => {
    this.#set(liveRecentTags(this.tags, live));
  };

  clear = (): void => {
    this.#set([]);
  };

  #set(tags: TagRef[]): void {
    if (sameRecentTags(tags, this.tags)) return;
    this.tags = tags;
    writeStored(STORAGE_KEY, { tags } satisfies Persisted);
  }
}

export const recentTags = new RecentTagsStore();
