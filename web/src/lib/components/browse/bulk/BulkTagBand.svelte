<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { listTags, type TagSummary } from '#lib/api/tags.ts';
  import Chip from '#lib/components/chips/Chip.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { nextStage, type Stage } from '#lib/browse/bulk/pickStage.ts';
  import { isManagedTag, toTagRef } from '#lib/browse/managedTags.ts';
  import { recentTags } from '#lib/stores/recentTags.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { mdiTagOutline } from '@mdi/js';
  import BulkPickList, { type PickItem } from './BulkPickList.svelte';

  let { onDone }: { onDone?: () => void } = $props();

  let tags = $state<TagSummary[]>([]);
  const stages = new SvelteMap<string, Stage>();

  const total = $derived(bulkScope.count);
  const counts = $derived.by(() => {
    const held = new Map<string, number>();
    for (const asset of bulkScope.picked) {
      for (const tag of asset.tags) held.set(tag.id, (held.get(tag.id) ?? 0) + 1);
    }
    return held;
  });
  const items = $derived<PickItem[]>(
    tags.map((tag) => ({ id: tag.id, label: tag.value, icon: mdiTagOutline }))
  );
  const recent = $derived(recentTags.tags.filter((tag) => tags.some((t) => t.id === tag.id)));

  function holding(id: string): number {
    return counts.get(id) ?? 0;
  }

  onMount(() => {
    listTags()
      .then((items) => {
        tags = items
          .filter((tag) => !isManagedTag(toTagRef(tag)))
          .sort((a, b) => a.value.localeCompare(b.value));
      })
      .catch((e: unknown) => toasts.fail('Failed to load tags', e));
  });

  function toggle(id: string): void {
    const next = nextStage(holding(id), total, stages.get(id));
    if (next) stages.set(id, next);
    else stages.delete(id);
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
        <Chip pressed={stages.get(tag.id) === 'add'} onPressedChange={() => toggle(tag.id)}>
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
