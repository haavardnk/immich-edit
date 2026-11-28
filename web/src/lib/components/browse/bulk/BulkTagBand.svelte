<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import Chip from '#lib/components/chips/Chip.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { MEMBERSHIP_CAP, heldCounts } from '#lib/browse/bulk/heldMembership.ts';
  import { nextStage, type Stage } from '#lib/browse/bulk/pickStage.ts';
  import { sourceId } from '#lib/browse/assetKey.ts';
  import { toTagRef, userTags } from '#lib/browse/managedTags.ts';
  import { loadTagMembership } from '#lib/browse/tagMembership.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { recentTags } from '#lib/stores/recentTags.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { tagCache } from '#lib/stores/tagCache.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { mdiTagOutline } from '@mdi/js';
  import BulkPickList, { type PickItem } from './BulkPickList.svelte';

  let { onDone }: { onDone?: () => void } = $props();

  const tags = $derived(userTags(library.tags).sort((a, b) => a.value.localeCompare(b.value)));
  const stages = new SvelteMap<string, Stage>();

  const sources = $derived([...new Set([...selection.selected].map(sourceId))]);
  const total = $derived(sources.length);
  const counts = $derived(heldCounts(sources, (id) => tagCache.get(id)));
  const items = $derived<PickItem[]>(
    tags.map((tag) => ({ id: tag.id, label: tag.value, icon: mdiTagOutline }))
  );
  const recent = $derived(recentTags.tags.filter((tag) => tags.some((t) => t.id === tag.id)));

  function holding(id: string): number {
    return counts?.get(id) ?? 0;
  }

  onMount(() => {
    library.ensureTags().catch((e: unknown) => toasts.fail('Failed to load tags', e));
  });

  $effect(() => {
    if (sources.length <= MEMBERSHIP_CAP) void loadTagMembership(sources);
  });

  function toggle(id: string): void {
    const next = nextStage(holding(id), total, stages.get(id));
    if (next) stages.set(id, next);
    else stages.delete(id);
  }

  function toggleAdd(id: string): void {
    if (stages.get(id) === 'add') stages.delete(id);
    else stages.set(id, 'add');
  }

  function refs(stage: Stage) {
    return tags.filter((tag) => stages.get(tag.id) === stage).map(toTagRef);
  }

  async function apply(): Promise<void> {
    if (!(await bulkActions.changeTags(refs('add'), refs('remove')))) return;
    stages.clear();
    onDone?.();
  }
</script>

{#snippet recentChips()}
  {#if recent.length > 0}
    <div class="section-label px-3 pt-3 pb-1 text-dark/65 touch:px-4">Recent</div>
    <div class="flex flex-wrap gap-2 px-3 pb-2 touch:px-4">
      {#each recent as tag (tag.id)}
        <Chip pressed={stages.get(tag.id) === 'add'} onPressedChange={() => toggleAdd(tag.id)}>
          {tag.value}
        </Chip>
      {/each}
    </div>
  {/if}
{/snippet}

<div
  class="max-h-96 overflow-y-auto border-t border-hairline touch:flex touch:max-h-none touch:flex-1 touch:flex-col touch:overflow-visible touch:border-t-0"
>
  <BulkPickList
    {items}
    {total}
    {holding}
    {stages}
    searchLabel="Search tags"
    allTitle="All tags"
    busy={bulkActions.metaBusy}
    lead={recentChips}
    onToggle={toggle}
    onApply={() => void apply()}
  />
</div>
