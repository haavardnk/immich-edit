<script lang="ts">
  import BrowseBarActions from '#lib/components/browse/BrowseBarActions.svelte';
  import BrowseToolbar from '#lib/components/browse/BrowseToolbar.svelte';
  import FilterChipRow from '#lib/components/browse/FilterChipRow.svelte';
  import SelectionAppBar from '#lib/components/browse/bulk/SelectionAppBar.svelte';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { photoCountBadge, photoCountLabel } from '#lib/browse/filterChips.ts';
  import type { PageParent } from '#lib/components/shell/pageParent.ts';
  import type { Snippet } from 'svelte';

  let {
    title,
    parent,
    totalCount,
    hiddenCount = 0,
    sortBasis = 'capture',
    bar,
    titleActions,
    onTrashRejected
  }: {
    title: string;
    parent?: PageParent;
    totalCount?: number;
    hiddenCount?: number;
    sortBasis?: 'capture' | 'edit' | null;
    bar?: Snippet;
    titleActions?: Snippet;
    onTrashRejected?: () => Promise<void>;
  } = $props();

  const count = $derived(
    totalCount === undefined ? undefined : photoCountLabel(totalCount, hiddenCount)
  );
  const badge = $derived(
    totalCount === undefined ? undefined : photoCountBadge(totalCount, hiddenCount)
  );
</script>

{#snippet selectionBar()}
  <SelectionAppBar />
{/snippet}

<PageHeader
  {badge}
  {title}
  {parent}
  {count}
  {bar}
  {titleActions}
  selecting={bulkScope.selecting ? selectionBar : undefined}
>
  {#snippet actions()}
    <BrowseToolbar {sortBasis} />
  {/snippet}
  {#snippet touchActions()}
    <BrowseBarActions />
  {/snippet}
</PageHeader>
<FilterChipRow {onTrashRejected} />
