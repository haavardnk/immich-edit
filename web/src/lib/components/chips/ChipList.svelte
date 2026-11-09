<script lang="ts" generics="T">
  import { chipRow, chipTone } from '#lib/components/chips/chipStyles.ts';
  import type { Snippet } from 'svelte';

  let {
    items,
    key,
    selected,
    limit = 8,
    label,
    chip,
    children
  }: {
    items: readonly T[];
    key: (item: T) => string;
    selected: (item: T) => boolean;
    limit?: number;
    label: string;
    chip: Snippet<[T]>;
    children?: Snippet;
  } = $props();

  let expanded = $state(false);

  const shown = $derived(
    expanded || items.length <= limit + 1
      ? items
      : items.filter((item, index) => index < limit || selected(item))
  );
  const hidden = $derived(items.length - shown.length);
</script>

<div role="group" aria-label={label} class={chipRow}>
  {#each shown as item (key(item))}
    {@render chip(item)}
  {/each}
  {#if hidden > 0}
    <button type="button" class={chipTone.more} onclick={() => (expanded = true)}>
      +{hidden} more
    </button>
  {/if}
  {@render children?.()}
</div>
