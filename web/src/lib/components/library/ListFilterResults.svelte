<script lang="ts" generics="T">
  import type { ListFilterState } from '#lib/browse/listFilter.svelte.ts';
  import type { Snippet } from 'svelte';

  let {
    filter,
    class: className = '',
    children
  }: {
    filter: ListFilterState<T>;
    class?: string;
    children: Snippet<[readonly T[], string]>;
  } = $props();
</script>

<div bind:this={filter.rows} class="contents">
  {@render children(filter.visible, filter.needle)}
</div>
{#if filter.available}
  <p
    role="status"
    class="text-xs text-dark/65 touch:text-label {filter.needle
      ? 'py-1 touch:py-2'
      : ''} {className}"
  >
    {filter.status}
  </p>
{/if}
