<script lang="ts">
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    disabled = false,
    onSelect,
    children
  }: {
    title?: string;
    disabled?: boolean;
    onSelect: () => void;
    children: Snippet;
  } = $props();
</script>

<Bits.Item
  {title}
  {disabled}
  {onSelect}
  class="flex cursor-pointer items-center px-3 py-1.5 text-xs outline-none select-none data-disabled:cursor-default data-disabled:opacity-40 data-highlighted:bg-white/10"
>
  {@render children()}
</Bits.Item>
