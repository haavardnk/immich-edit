<script lang="ts" generics="T">
  import type { ListFilterState } from '#lib/browse/listFilter.svelte.ts';
  import SearchField from '#lib/components/SearchField.svelte';

  let {
    filter,
    compact = false,
    class: className = ''
  }: {
    filter: ListFilterState<T>;
    compact?: boolean;
    class?: string;
  } = $props();
</script>

{#if filter.available}
  <div class={className}>
    <SearchField
      bind:value={filter.query}
      label={filter.label}
      {compact}
      onkeydown={(event) => filter.focusFirstRow(event)}
    />
  </div>
{/if}
