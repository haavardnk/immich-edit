<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import ViewOptions from '#lib/components/browse/ViewOptions.svelte';
  import { activeFilterChips } from '#lib/browse/filterChips.ts';
  import { openFilterPanel } from '#lib/browse/filterPanel.ts';
  import { idName } from '#lib/browse/idOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal, mdiFilterOutline } from '@mdi/js';

  let { sortBasis }: { sortBasis: 'capture' | 'edit' | null } = $props();

  let open = $state(false);

  const filterCount = $derived(activeFilterChips(browseControls.filters, idName).length);
  const filtersLabel = $derived(filterCount > 0 ? `Filters (${filterCount} active)` : 'Filters');
</script>

<div class="relative">
  <IconButton
    id="browse-filters-toggle"
    size="medium"
    variant="ghost"
    color={filterCount > 0 ? 'primary' : 'secondary'}
    shape="round"
    class="size-11"
    icon={mdiFilterOutline}
    aria-label={filtersLabel}
    onclick={openFilterPanel}
  />
  {#if filterCount > 0}
    <span
      class="pointer-events-none absolute top-1 right-1 min-w-4 rounded-full bg-primary px-1 text-center text-[10px] leading-4 text-light"
      aria-hidden="true"
    >
      {filterCount}
    </span>
  {/if}
</div>
<Popover
  {open}
  align="end"
  onOpenChange={(v) => (open = v)}
  appearance="panel"
  title="View options"
  contentClass="flex min-w-64 flex-col gap-3 p-3"
  sheetClass="flex flex-col gap-4 px-4 py-3"
>
  {#snippet trigger(popoverProps)}
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      class="size-11"
      icon={mdiDotsHorizontal}
      aria-label="View options"
      {...popoverProps}
    />
  {/snippet}
  <ViewOptions {sortBasis} />
</Popover>
