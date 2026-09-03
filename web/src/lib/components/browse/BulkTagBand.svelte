<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { addTagToAsset, listTags, removeTagFromAsset, type TagSummary } from '$lib/api/tags';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import { isManagedTag, toTagRef } from '$lib/reject';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { selection } from '$lib/stores/selection.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';

  type RunPool = <T>(items: T[], limit: number, fn: (item: T) => Promise<void>) => Promise<void>;

  let { runPool, busy = $bindable() }: { runPool: RunPool; busy: boolean } = $props();

  let tags = $state<TagSummary[]>([]);
  let tagsLoaded = $state(false);
  let chosenTags = $state<string[]>([]);

  let metaBusy = $derived(busy);
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
    if (busy || chosenTags.length === 0) return;
    if (!(await metadataConsent.gate())) return;
    busy = true;
    const ids = [...selection.selected];
    let failed = 0;
    const pairs = ids.flatMap((id) => chosenTags.map((tagId) => ({ id, tagId })));
    await runPool(pairs, 6, async ({ id, tagId }) => {
      try {
        await (add ? addTagToAsset(tagId, id) : removeTagFromAsset(tagId, id));
      } catch {
        failed += 1;
      }
    });
    busy = false;
    if (failed > 0) {
      toasts.push('warn', `${failed} tag updates failed`, 6000);
    } else {
      toasts.push(
        'success',
        `${add ? 'Added' : 'Removed'} tags on ${ids.length} asset${ids.length === 1 ? '' : 's'}`,
        4000
      );
    }
    chosenTags = [];
  }
</script>

<div class="flex flex-col gap-2 border-t px-4 py-3">
  <span class="px-1 text-xs text-dark/60">
    Pick tags, then apply them to the {selection.count} selected asset{selection.count === 1
      ? ''
      : 's'}
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
      {#if chosenTagItems.length > 0}
        <div class="mt-1 flex flex-wrap gap-1">
          {#each chosenTagItems as tag (tag.id)}
            <span
              class="inline-flex h-8 items-center gap-1 rounded-full bg-light-200 pl-3 text-xs text-dark dark:bg-primary-200 dark:text-white"
            >
              {tag.value}
              <IconButton
                size="tiny"
                variant="ghost"
                color="secondary"
                icon={mdiClose}
                aria-label="Remove {tag.value}"
                onclick={() => (chosenTags = chosenTags.filter((id) => id !== tag.id))}
              />
            </span>
          {/each}
        </div>
      {/if}
    </div>
    <div class="grid grid-cols-2 gap-2">
      <Button
        size="tiny"
        color="primary"
        class="h-7 whitespace-nowrap"
        disabled={metaBusy || chosenTags.length === 0}
        onclick={() => void applyTags(true)}
      >
        Add to selected
      </Button>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 whitespace-nowrap"
        disabled={metaBusy || chosenTags.length === 0}
        onclick={() => void applyTags(false)}
      >
        Remove from selected
      </Button>
    </div>
  </div>
</div>
