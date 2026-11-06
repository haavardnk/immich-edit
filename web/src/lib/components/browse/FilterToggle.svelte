<script lang="ts">
  import { activeFilterChips } from '#lib/browse/filterChips.ts';
  import { openFilterPanel } from '#lib/browse/filterPanel.ts';
  import { idName } from '#lib/browse/idOptions.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import { mdiFilterOutline } from '@mdi/js';

  let { size = 'medium', class: className = '' }: { size?: 'small' | 'medium'; class?: string } =
    $props();

  const filterCount = $derived(activeFilterChips(browseControls.filters, idName).length);
  const filtersLabel = $derived(filterCount > 0 ? `Filters (${filterCount} active)` : 'Filters');
  const large = $derived(size === 'medium');
</script>

<div class="relative {className}">
  <IconButton
    id="browse-filters-toggle"
    {size}
    variant="ghost"
    color={filterCount > 0 ? 'primary' : 'secondary'}
    shape={large ? 'round' : undefined}
    class={large ? 'size-11' : ''}
    icon={mdiFilterOutline}
    title={hint('Filters', 'toggleFilters')}
    aria-label={filtersLabel}
    onclick={openFilterPanel}
  />
  {#if filterCount > 0}
    <span
      class="pointer-events-none absolute rounded-full bg-primary text-center font-bold text-light ring-2 ring-light {large
        ? 'top-1.25 right-0.75 h-4.25 min-w-4.25 px-1 text-[11px] leading-4.25'
        : '-top-1 -right-1 h-3.5 min-w-3.5 px-0.5 text-[9px] leading-3.5'}"
      aria-hidden="true"
    >
      {filterCount}
    </span>
  {/if}
</div>
