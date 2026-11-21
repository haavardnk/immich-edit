<script lang="ts" generics="T">
  import type { ListFilterState } from '#lib/browse/listFilter.svelte.ts';
  import SearchField from '#lib/components/SearchField.svelte';
  import type { SearchFieldSize } from '#lib/components/searchFieldSize.ts';

  let {
    filter,
    size = 'default',
    class: className = ''
  }: {
    filter: ListFilterState<T>;
    size?: SearchFieldSize;
    class?: string;
  } = $props();
</script>

{#if filter.available}
  <div class={className}>
    <SearchField
      bind:value={filter.query}
      label={filter.label}
      {size}
      onkeydown={(event) => filter.focusFirstRow(event)}
    />
  </div>
{/if}
