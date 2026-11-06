<script lang="ts" generics="T">
  import TextInput from '#lib/components/TextInput.svelte';
  import {
    filterByName,
    listFilterShown,
    listFilterStatus,
    type ListNoun
  } from '#lib/browse/listFilter.ts';
  import { Icon } from '@immich/ui';
  import { mdiMagnify } from '@mdi/js';
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
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      if (query) query = '';
      else input?.blur();
      return;
    }
    if (event.key !== 'ArrowDown') return;
    const first = rows?.querySelector<HTMLElement>('a[href], button:not([disabled])');
    if (!first) return;
    event.preventDefault();
    first.focus();
  }
</script>

{#snippet searchIcon(_disabled: boolean)}
  <Icon icon={mdiMagnify} size="60%" class="text-dark/65" aria-hidden />
{/snippet}

{#if available}
  <div class="py-1 {className}">
    <TextInput
      bind:ref={input}
      bind:value={query}
      type="text"
      size="tiny"
      autocomplete="off"
      placeholder={label}
      aria-label={label}
      leadingIcon={searchIcon}
      class="rounded-lg bg-light-100! ring-1 ring-primary/20! focus-within:bg-light-100! focus-within:ring-primary! [&_input]:h-7.5 [&_input]:text-[13px] touch:[&_input]:h-11 touch:[&_input]:text-[15px]"
      {onkeydown}
    />
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
