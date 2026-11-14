<script lang="ts">
  import FilterCount from '#lib/components/browse/filters/FilterCount.svelte';
  import { activeFilterCount } from '#lib/browse/filterChips.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';

  const count = $derived(activeFilterCount(browseControls.filters));
</script>

<div class="flex h-11 items-center gap-2 border-y border-hairline pr-1 pl-4">
  <h3 class="text-xs font-semibold tracking-[.06em] text-dark/65 uppercase">Filters</h3>
  <FilterCount {count} />
  <span class="flex-1"></span>
  <Button
    size="small"
    variant="ghost"
    color="primary"
    shape="round"
    class="h-9 bg-transparent px-3 text-sm font-medium"
    leadingIcon={mdiRestore}
    aria-label="Reset all filters"
    disabled={count === 0}
    onclick={() => browseControls.resetFilters()}
  >
    Reset
  </Button>
</div>
