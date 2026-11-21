import {
  createTag,
  deleteTag,
  updateTag,
  type NewTag,
  type TagPatch,
  type TagSummary
} from '#lib/api/tags.ts';
import { library } from '#lib/stores/library.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { tagCache } from '#lib/stores/tagCache.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

class TagStore {
  create = async (tag: NewTag): Promise<TagSummary | null> => {
    try {
      const created = await createTag(tag);
      library.putTag(created);
      return created;
    } catch (e) {
      toasts.fail('Create tag', e);
      return null;
    }
  };

  update = async (id: string, patch: TagPatch): Promise<boolean> => {
    try {
      await updateTag(id, patch);
    } catch (e) {
      toasts.fail('Save tag details', e);
      return false;
    }
    await this.changed();
    return true;
  };

  remove = async (tag: TagSummary, deleted?: () => Promise<void>): Promise<boolean> => {
    try {
      await deleteTag(tag.id);
    } catch (e) {
      toasts.fail('Delete tag', e);
      return false;
    }
    await deleted?.();
    await this.changed();
    return true;
  };

  private async changed(): Promise<void> {
    tagCache.clear();
    libraryChanged();
    await library.refreshTags().catch(() => undefined);
  }
}

export const tag = new TagStore();
