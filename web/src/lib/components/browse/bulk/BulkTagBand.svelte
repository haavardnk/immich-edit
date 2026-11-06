<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '@immich/ui';
  import { listTags, type TagSummary } from '#lib/api/tags.ts';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import ChosenChips from './ChosenChips.svelte';
  import { isManagedTag, toTagRef } from '#lib/browse/managedTags.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';

  let tags = $state<TagSummary[]>([]);
  let tagsLoaded = $state(false);
  let chosenTags = $state<string[]>([]);

  let chosenTagItems = $derived(tags.filter((tag) => chosenTags.includes(tag.id)));

  onMount(() => {
    if (tagsLoaded) return;
    tagsLoaded = true;
    listTags()
      .then((items) => {
        tags = items.filter((tag) => !isManagedTag(toTagRef(tag)));
      })
      .catch(() => (tagsLoaded = false));
  });

  async function applyTags(add: boolean): Promise<void> {
    if (chosenTags.length === 0) return;
    if (await bulkActions.setTags(chosenTagItems.map(toTagRef), add)) chosenTags = [];
  }
</script>

<div class="flex flex-col gap-2 border-t px-4 py-3 touch:border-t-0 touch:px-0">
  <span class="px-1 text-xs text-dark/60">
    Pick tags, then apply them to the {countOf(bulkScope.count, 'selected asset')}
  </span>
  <div class="flex flex-col gap-2 sm:flex-row sm:items-end">
    <div class="min-w-0 flex-1 sm:min-w-55">
      <SearchableSelect
        options={tags}
        bind:selected={chosenTags}
        getId={(t) => t.id}
        getLabel={(t) => t.value}
        placeholder="Choose tags…"
        color="neutral"
        side="top"
      />
      <ChosenChips
        items={chosenTagItems.map((tag) => ({ id: tag.id, label: tag.value }))}
        onRemove={(id) => (chosenTags = chosenTags.filter((tagId) => tagId !== id))}
      />
    </div>
    <div class="grid grid-cols-2 gap-2">
      <Button
        size="tiny"
        color="primary"
        class="h-7 whitespace-nowrap touch:h-11"
        disabled={bulkActions.metaBusy || chosenTags.length === 0}
        onclick={() => void applyTags(true)}
      >
        Add to selected
      </Button>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 whitespace-nowrap touch:h-11"
        disabled={bulkActions.metaBusy || chosenTags.length === 0}
        onclick={() => void applyTags(false)}
      >
        Remove from selected
      </Button>
    </div>
  </div>
</div>
