<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';

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

{#if layout.mode === 'touch'}
  <MenuRow {disabled} onclick={onSelect}>
    <span class="min-w-0 flex-1 truncate">{@render children()}</span>
  </MenuRow>
{:else}
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
{/if}
