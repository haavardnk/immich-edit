<script lang="ts">
  import BrowseBarActions from '#lib/components/browse/BrowseBarActions.svelte';
  import BrowseToolbar from '#lib/components/browse/BrowseToolbar.svelte';
  import FilterChipRow from '#lib/components/browse/FilterChipRow.svelte';
  import SelectionAppBar from '#lib/components/browse/bulk/SelectionAppBar.svelte';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { photoCountLabel } from '#lib/browse/filterChips.ts';
  import type { Snippet } from 'svelte';

  let {
    title,
    totalCount,
    hiddenCount = 0,
    sortBasis = 'capture',
    bar,
    onTrashRejected
  }: {
    title: string;
    totalCount?: number;
    hiddenCount?: number;
    sortBasis?: 'capture' | 'edit' | null;
    bar?: Snippet;
    onTrashRejected?: () => Promise<void>;
  } = $props();

  const count = $derived(
    totalCount === undefined ? undefined : photoCountLabel(totalCount, hiddenCount)
  );
</script>

{#snippet selectionBar()}
  <SelectionAppBar />
{/snippet}

<PageHeader {title} {count} {bar} selecting={bulkScope.selecting ? selectionBar : undefined}>
  {#snippet actions()}
    <BrowseToolbar {sortBasis} />
  {/snippet}
  {#snippet touchActions()}
    <BrowseBarActions />
  {/snippet}
</PageHeader>
<FilterChipRow {onTrashRejected} />
