<script lang="ts">
  import { chipTone, type ChipTone } from '#lib/components/chips/chipStyles.ts';
  import { Icon, Tooltip } from '@immich/ui';
  import { Toggle } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    pressed,
    onPressedChange,
    tone = 'value',
    icon,
    iconClass = '',
    tooltip,
    label,
    disabled = false,
    class: className = '',
    children
  }: {
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
    tone?: ChipTone;
    icon?: string;
    iconClass?: string;
    tooltip?: string;
    label?: string;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
  } = $props();
</script>

{#snippet chip(props: Record<string, unknown>)}
  <Toggle.Root
    {...props}
    bind:pressed={() => pressed, onPressedChange}
    {disabled}
    aria-label={label}
    class="{chipTone[tone]} {className}"
  >
    {#if icon}
      <Icon {icon} size="1.15em" class="shrink-0 {iconClass}" aria-hidden="true" />
    {/if}
    {@render children?.()}
  </Toggle.Root>
{/snippet}

{#if tooltip}
  <Tooltip text={tooltip}>
    {#snippet child({ props })}
      {@render chip(props)}
    {/snippet}
  </Tooltip>
{:else}
  {@render chip({})}
{/if}
