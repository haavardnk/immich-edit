<script lang="ts">
  import BrowseBarActions from '#lib/components/browse/BrowseBarActions.svelte';
  import BrowseToolbar from '#lib/components/browse/BrowseToolbar.svelte';
  import FilterChipRow from '#lib/components/browse/FilterChipRow.svelte';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
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

<PageHeader {title} {count} {bar}>
  {#snippet actions()}
    <BrowseToolbar {sortBasis} />
  {/snippet}
  {#snippet compactActions()}
    <BrowseBarActions {sortBasis} />
  {/snippet}
</PageHeader>
<FilterChipRow {onTrashRejected} />
