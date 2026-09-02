<script lang="ts">
  import { library } from '$lib/stores/library.svelte';
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import type { AlbumSummary } from '$lib/types/album';
  import type { TagSummary } from '$lib/api/tags';
  import { isManagedTag, toTagRef } from '$lib/reject';
  import type { ExportForm } from './settings';
  import type { StackPrimary } from '$lib/api/export';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();

  const availableTags = $derived(library.tags.filter((tag) => !isManagedTag(toTagRef(tag))));
  const selectedAlbums = $derived(
    library.albums
      .filter((album) => form.albumIds.includes(album.id))
      .map((album) => ({ id: album.id, label: album.albumName }))
  );
  const selectedTags = $derived(
    availableTags
      .filter((tag) => form.tagIds.includes(tag.id))
      .map((tag) => ({ id: tag.id, label: tag.value || tag.name }))
  );
</script>

{#snippet pills(items: { id: string; label: string }[], remove: (id: string) => void)}
  {#if items.length > 0}
    <div class="mt-1 flex flex-wrap gap-1">
      {#each items as item (item.id)}
        <div class="group flex">
          <span
            class="inline-flex h-6 items-center rounded-s-full bg-primary ps-2.5 pe-1 text-[11px] whitespace-nowrap text-gray-800"
          >
            {item.label}
          </span>
          <IconButton
            size="tiny"
            variant="filled"
            color="primary"
            shape="rectangle"
            class="size-6 rounded-s-none rounded-e-full"
            icon={mdiClose}
            title="Remove {item.label}"
            aria-label="Remove {item.label}"
            onclick={() => remove(item.id)}
          />
        </div>
      {/each}
    </div>
  {/if}
{/snippet}

<div class="flex flex-col gap-1">
  <div class="panel-row min-h-7 items-start">
    <span class="editor-compact-label leading-7 select-none">Albums</span>
    <div class="col-span-2 col-start-2 min-w-0">
      <SearchableSelect
        compact
        color="neutral"
        options={library.albums}
        bind:selected={form.albumIds}
        getId={(a: AlbumSummary) => a.id}
        getLabel={(a: AlbumSummary) => a.albumName}
        placeholder="Add album…"
      />
      {@render pills(
        selectedAlbums,
        (id) => (form.albumIds = form.albumIds.filter((v) => v !== id))
      )}
    </div>
  </div>

  <div class="panel-row min-h-7 items-start">
    <span class="editor-compact-label leading-7 select-none">Tags</span>
    <div class="col-span-2 col-start-2 min-w-0">
      <SearchableSelect
        compact
        color="neutral"
        options={availableTags}
        bind:selected={form.tagIds}
        getId={(t: TagSummary) => t.id}
        getLabel={(t: TagSummary) => t.value || t.name}
        placeholder="Add tag…"
      />
      {@render pills(selectedTags, (id) => (form.tagIds = form.tagIds.filter((v) => v !== id)))}
    </div>
  </div>

  <div class="grid grid-cols-2 gap-x-2">
    <CheckboxRow
      label="Mark as favorite"
      checked={form.favorite}
      onChange={(v) => (form.favorite = v)}
    />
    <CheckboxRow
      label="Stack with original"
      checked={form.stackWithOriginal}
      onChange={(v) => (form.stackWithOriginal = v)}
    />
  </div>

  {#if form.stackWithOriginal}
    <RadioGroup.Root
      bind:value={() => form.stackPrimary, (v) => (form.stackPrimary = v as StackPrimary)}
      orientation="horizontal"
      aria-label="Stack primary"
      class="{segmentedControlClass} ml-5 self-start"
    >
      <RadioGroup.Item value="edited" class="{segmentedRadioItemClass} px-2">
        Edit primary
      </RadioGroup.Item>
      <RadioGroup.Item value="original" class="{segmentedRadioItemClass} px-2">
        Original primary
      </RadioGroup.Item>
    </RadioGroup.Root>
  {/if}
</div>
