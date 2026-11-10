<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Icon } from '@immich/ui';

  let {
    title,
    detail,
    icon,
    disabled = false,
    onSelect,
    children
  }: {
    title?: string;
    detail?: string;
    icon?: string;
    disabled?: boolean;
    onSelect: () => void;
    children: Snippet;
  } = $props();
</script>

{#snippet trailing()}
  {#if detail}
    <span class="ml-auto shrink-0 pl-4 text-[10px] text-dark/65 touch:pl-0 touch:text-[13px]">
      {detail}
    </span>
  {/if}
{/snippet}

{#if layout.mode === 'touch'}
  <MenuRow {icon} {disabled} onclick={onSelect}>
    <span class="min-w-0 flex-1 truncate">{@render children()}</span>
    {@render trailing()}
  </MenuRow>
{:else}
  <ElementTooltip text={title}>
    {#snippet child({ props })}
      <Bits.Item
        {...props}
        {disabled}
        {onSelect}
        class="flex cursor-pointer items-center gap-2.5 px-3 py-1.5 text-xs outline-none select-none data-disabled:cursor-default data-disabled:opacity-40 data-highlighted:bg-light-950/10"
      >
        {#if icon}
          <Icon {icon} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
        {/if}
        {@render children()}
        {@render trailing()}
      </Bits.Item>
    {/snippet}
  </ElementTooltip>
{/if}
