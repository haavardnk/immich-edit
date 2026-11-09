<script lang="ts">
  import Chip from '#lib/components/chips/Chip.svelte';
  import { chipRow } from '#lib/components/chips/chipStyles.ts';
  import SizeField from '#lib/components/browse/filters/SizeField.svelte';
  import { SIZE_PRESETS, sizePreset, type SizePreset } from '#lib/browse/fileSize.ts';
  import { browseControls, type BrowseFilters } from '#lib/stores/browseControls.svelte.ts';

  let customFor = $state.raw<BrowseFilters | null>(null);

  const min = $derived(browseControls.filters.minSize);
  const max = $derived(browseControls.filters.maxSize);
  const preset = $derived(sizePreset(min, max));
  const custom = $derived(
    customFor === browseControls.filters || ((min !== null || max !== null) && !preset)
  );

  function setSize(next: SizePreset | null): void {
    customFor = null;
    browseControls.filters.minSize = next?.min ?? null;
    browseControls.filters.maxSize = next?.max ?? null;
  }

  function pickCustom(pressed: boolean): void {
    if (pressed) customFor = browseControls.filters;
    else setSize(null);
  }
</script>

<div role="group" aria-label="File size" class={chipRow}>
  {#each SIZE_PRESETS as entry (entry.id)}
    <Chip
      pressed={!custom && preset?.id === entry.id}
      onPressedChange={(pressed) => setSize(pressed ? entry : null)}
    >
      {entry.label}
    </Chip>
  {/each}
  <Chip pressed={custom} onPressedChange={pickCustom}>Custom</Chip>
</div>
{#if custom}
  <div class="grid grid-cols-2 gap-2">
    <SizeField key="minSize" label="Min size (MB)" />
    <SizeField key="maxSize" label="Max size (MB)" />
  </div>
{/if}
