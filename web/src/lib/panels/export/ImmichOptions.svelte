<script lang="ts">
  import { library } from '$lib/stores/library.svelte';
  import MultiSelect from '$lib/components/MultiSelect.svelte';
  import type { AlbumSummary } from '$lib/types/album';
  import type { TagSummary } from '$lib/api/tags';
  import { isManagedTag, toTagRef } from '$lib/reject';
  import type { ExportForm } from './settings';

  let {
    form = $bindable<ExportForm>(),
    radioName = 'stackPrimary',
  }: { form: ExportForm; radioName?: string } = $props();
</script>

<label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
  <input type="checkbox" class="checkbox checkbox-xs" bind:checked={form.favorite} />
  Mark as favorite
</label>

<label class="flex items-center gap-2 text-xs text-immich-dark-fg/80 select-none cursor-pointer">
  <input type="checkbox" class="checkbox checkbox-xs" bind:checked={form.stackWithOriginal} />
  Stack with original
</label>

{#if form.stackWithOriginal}
  <div class="flex gap-3 pl-6">
    <label class="flex items-center gap-1 text-[11px] text-immich-dark-fg/80 cursor-pointer">
      <input
        type="radio"
        class="radio radio-xs"
        name={radioName}
        value="edited"
        bind:group={form.stackPrimary}
      />
      Edit primary
    </label>
    <label class="flex items-center gap-1 text-[11px] text-immich-dark-fg/80 cursor-pointer">
      <input
        type="radio"
        class="radio radio-xs"
        name={radioName}
        value="original"
        bind:group={form.stackPrimary}
      />
      Original primary
    </label>
  </div>
{/if}

<div class="flex flex-col gap-1">
  <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Albums</span>
  <MultiSelect
    options={library.albums}
    bind:selected={form.albumIds}
    getId={(a: AlbumSummary) => a.id}
    getLabel={(a: AlbumSummary) => a.albumName}
    placeholder="Add album…"
  />
</div>

<div class="flex flex-col gap-1">
  <span class="text-[11px] leading-none text-immich-dark-fg/60 select-none">Tags</span>
  <MultiSelect
    options={library.tags.filter((t) => !isManagedTag(toTagRef(t)))}
    bind:selected={form.tagIds}
    getId={(t: TagSummary) => t.id}
    getLabel={(t: TagSummary) => t.value || t.name}
    placeholder="Add tag…"
  />
</div>
