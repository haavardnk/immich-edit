<script lang="ts" generics="T">
  import SearchField from '#lib/components/SearchField.svelte';
  import {
    filterByName,
    listFilterShown,
    listFilterStatus,
    type ListNoun
  } from '#lib/browse/listFilter.ts';
  import { firstFocusable } from '#lib/utils/focusable.ts';
  import type { Snippet } from 'svelte';

  let {
    items,
    nameOf,
    noun,
    class: className = '',
    children
  }: {
    items: readonly T[];
    nameOf: (item: T) => string;
    noun: ListNoun;
    class?: string;
    children: Snippet<[readonly T[], string]>;
  } = $props();

  let query = $state('');
  let input = $state<HTMLInputElement | null>(null);
  let rows = $state<HTMLElement | null>(null);

  const available = $derived(listFilterShown(items.length));
  const needle = $derived(available ? query.trim() : '');
  const visible = $derived(needle ? filterByName(items, needle, nameOf) : items);
  const label = $derived(`Filter ${noun.many}`);

  function onkeydown(event: KeyboardEvent): void {
    if (event.key !== 'ArrowDown') return;
    const first = rows ? firstFocusable(rows) : null;
    if (!first) return;
    event.preventDefault();
    first.focus();
  }
</script>

{#if available}
  <div class="py-1 {className}">
    <SearchField bind:ref={input} bind:value={query} {label} {onkeydown} />
  </div>
{/if}
<div bind:this={rows} class="contents">
  {@render children(visible, needle)}
</div>
{#if available}
  <p
    role="status"
    class="text-[11.5px] text-dark/65 touch:text-[13px] {needle
      ? 'py-1 touch:py-2'
      : ''} {className}"
  >
    {needle ? listFilterStatus(visible.length, items.length, noun) : ''}
  </p>
{/if}
