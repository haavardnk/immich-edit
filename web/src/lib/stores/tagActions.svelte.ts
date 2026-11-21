import type { TagSummary } from '#lib/api/tags.ts';
import type { TagAction } from '#lib/browse/tagActions.ts';
import { tagDetailsPatch } from '#lib/browse/tagDetails.ts';
import { userTags } from '#lib/browse/managedTags.ts';
import { ancestorKeys, buildTagTree, descendantCount, type TagNode } from '#lib/browse/tagTree.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { library } from '#lib/stores/library.svelte.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { tag } from '#lib/stores/tag.svelte.ts';
import { tagTreeOpen } from '#lib/stores/tagTreeOpen.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { goto } from '$app/navigation';
import { page } from '$app/state';

export type TagDetailsField = 'name' | 'color';

export type TagDetailsRequest =
  | { kind: 'edit'; tag: TagSummary; subtags: number; focus: TagDetailsField }
  | { kind: 'create'; parentId: string | null };

export interface TagConfirmRequest {
  tag: TagSummary;
  subtags: number;
}

export interface TagDetailsInput {
  name: string;
  color: string | null;
  parentId: string | null;
}

function tagPath(id: string): string {
  return `/tags/${id}`;
}

function viewing(target: TagSummary): boolean {
  const current = page.url.pathname.startsWith('/tags/')
    ? library.tags.find((entry) => entry.id === page.params.id)
    : undefined;
  if (!current) return false;
  return current.id === target.id || current.value.startsWith(`${target.value}/`);
}

class TagActionsStore {
  details = $state<TagDetailsRequest | null>(null);
  confirming = $state<TagConfirmRequest | null>(null);
  saving = $state(false);

  run = async (action: TagAction, node: TagNode): Promise<void> => {
    const target = node.tag;
    if (!target) return;
    if (action === 'open') {
      await goto(tagPath(target.id));
      return;
    }
    if (!(await metadataConsent.gate())) return;
    const subtags = descendantCount(node);
    if (action === 'subtag') this.details = { kind: 'create', parentId: target.id };
    else if (action === 'delete') this.confirming = { tag: target, subtags };
    else
      this.details = {
        kind: 'edit',
        tag: target,
        subtags,
        focus: action === 'color' ? 'color' : 'name'
      };
  };

  create = async (parentId: string | null = null): Promise<void> => {
    if (!(await metadataConsent.gate())) return;
    this.details = { kind: 'create', parentId };
  };

  closeDetails = (): void => {
    this.details = null;
  };

  save = async (input: TagDetailsInput): Promise<void> => {
    const request = this.details;
    if (!request || this.saving) return;
    this.saving = true;
    try {
      const done =
        request.kind === 'edit' ? await this.update(request.tag, input) : await this.add(input);
      if (done && this.details === request) this.details = null;
    } finally {
      this.saving = false;
    }
  };

  confirm = async (): Promise<void> => {
    const request = this.confirming;
    if (!request) return;
    this.confirming = null;
    const leaving = viewing(request.tag);
    const removed = await tag.remove(request.tag, async () => {
      if (leaving) await goto('/tags');
    });
    if (removed) toasts.push('success', `Deleted ${request.tag.name}`);
  };

  cancel = (): void => {
    this.confirming = null;
  };

  private async update(target: TagSummary, input: TagDetailsInput): Promise<boolean> {
    const patch = tagDetailsPatch(target, input.name, input.color, immichCapabilities.tagRename);
    return patch ? tag.update(target.id, patch) : true;
  }

  private async add(input: TagDetailsInput): Promise<boolean> {
    const created = await tag.create({
      name: input.name.trim(),
      ...(input.parentId ? { parentId: input.parentId } : {}),
      ...(input.color ? { color: input.color } : {})
    });
    if (!created) return false;
    tagTreeOpen.openAll(ancestorKeys(buildTagTree(userTags(library.tags)), created.id) ?? []);
    return true;
  }
}

export const tagActions = new TagActionsStore();
