<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
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

<ElementTooltip text={title}>
  {#snippet child({ props })}
    <Bits.Item
      {...props}
      {disabled}
      {onSelect}
      class="flex cursor-pointer items-center px-3 py-1.5 text-xs outline-none select-none data-disabled:cursor-default data-disabled:opacity-40 data-highlighted:bg-light-950/10"
    >
      {@render children()}
    </Bits.Item>
  {/snippet}
</ElementTooltip>
