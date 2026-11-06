<script lang="ts">
  import type { TagSummary } from '#lib/api/tags.ts';
  import { toTagRef } from '#lib/browse/managedTags.ts';
  import { recentTagSlot } from '#lib/browse/recentTags.ts';
  import { buildTagTree, hasExactTag, matchTags } from '#lib/browse/tagTree.ts';
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import AppliedTags from '#lib/components/tags/AppliedTags.svelte';
  import RecentTagChips from '#lib/components/tags/RecentTagChips.svelte';
  import TagMatches from '#lib/components/tags/TagMatches.svelte';
  import TagTree from '#lib/components/tags/TagTree.svelte';
  import { isKeybind } from '#lib/shortcuts/match.ts';
  import { recentTags } from '#lib/stores/recentTags.svelte.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { Icon } from '@immich/ui';
  import { mdiMagnify } from '@mdi/js';
  import type { Attachment } from 'svelte/attachments';

  let {
    all,
    applied,
    showApplied,
    numbered,
    disabled = false,
    autofocus = false,
    onAdd,
    onRemove,
    onCreate
  }: {
    all: TagSummary[];
    applied: TagRef[];
    showApplied: boolean;
    numbered: boolean;
    disabled?: boolean;
    autofocus?: boolean;
    onAdd: (tag: TagRef) => void | Promise<void>;
    onRemove: (tagId: string) => void | Promise<void>;
    onCreate: (value: string) => TagRef | null | Promise<TagRef | null>;
  } = $props();

  const allId = $props.id();

  let query = $state('');
  let list = $state<HTMLElement | null>(null);

  const appliedIds = $derived(new Set(applied.map((tag) => tag.id)));
  const searching = $derived(query.trim() !== '');
  const tree = $derived(buildTagTree(all));
  const matches = $derived(matchTags(all, query));
  const canCreate = $derived(searching && !hasExactTag(all, query));

  function set(tag: TagRef, on: boolean): void {
    if (on) void onAdd(tag);
    else void onRemove(tag.id);
  }

  function pick(tag: TagSummary, on: boolean): void {
    const ref = toTagRef(tag);
    if (on) recentTags.record(ref);
    set(ref, on);
  }

  async function create(): Promise<void> {
    const value = query.trim();
    if (!value || disabled) return;
    query = '';
    const tag = await onCreate(value);
    if (tag) recentTags.record(tag);
  }

  function inputKey(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      const first = matches[0];
      if (first && !disabled) {
        if (!appliedIds.has(first.id)) pick(first, true);
        query = '';
      } else if (canCreate) void create();
    } else if (event.key === 'ArrowDown') {
      const box = list?.querySelector<HTMLElement>('button[role="checkbox"]:not([disabled])');
      if (!box) return;
      event.preventDefault();
      box.focus();
    }
  }

  const recentKeys: Attachment<HTMLElement> = (node) => {
    const onKey = (event: KeyboardEvent): void => {
      if (!isKeybind(event, 'recentTag')) return;
      const slot = recentTagSlot(event.code);
      const tag = slot === null ? undefined : recentTags.tags[slot];
      if (!tag) return;
      event.preventDefault();
      event.stopPropagation();
      if (!disabled) void onAdd(tag);
    };
    node.addEventListener('keydown', onKey);
    return () => node.removeEventListener('keydown', onKey);
  };
</script>

{#snippet searchIcon()}
  <Icon icon={mdiMagnify} size="60%" aria-hidden="true" />
{/snippet}

<div class="flex flex-col pb-2 text-dark" {@attach recentKeys}>
  <div class="sticky top-0 z-10 bg-light-100 px-3 pt-3 pb-2 touch:bg-light-50 touch:px-4">
    <TextInput
      bind:value={query}
      {autofocus}
      size="small"
      placeholder="Add tags…"
      aria-label="Add tags…"
      leadingIcon={searchIcon}
      onkeydown={inputKey}
    />
  </div>
  {#if showApplied && applied.length > 0}
    <AppliedTags tags={applied} {disabled} onRemove={(id) => void onRemove(id)} />
  {/if}
  {#if !searching && recentTags.tags.length > 0}
    <RecentTagChips
      tags={recentTags.tags}
      applied={appliedIds}
      {numbered}
      {disabled}
      onToggle={set}
    />
  {/if}
  <section aria-labelledby={allId} bind:this={list}>
    <SectionHeading id={allId} title={searching ? 'Matches' : 'All tags'} level={3} />
    {#if searching}
      <TagMatches
        {matches}
        applied={appliedIds}
        {query}
        {canCreate}
        {disabled}
        onToggle={pick}
        onCreate={() => void create()}
      />
    {:else if tree.length > 0}
      <TagTree nodes={tree} applied={appliedIds} {disabled} onToggle={pick} />
    {:else}
      <p class="flex h-8 items-center px-3 text-xs text-muted touch:h-11 touch:px-4">No tags yet</p>
    {/if}
  </section>
</div>
