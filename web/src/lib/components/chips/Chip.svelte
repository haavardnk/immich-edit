<script lang="ts">
  import { chipTone, type ChipTone } from '#lib/components/chips/chipStyles.ts';
  import { Icon } from '@immich/ui';
  import { Toggle } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    pressed,
    onPressedChange,
    tone = 'value',
    icon,
    label,
    disabled = false,
    class: className = '',
    children
  }: {
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
    tone?: ChipTone;
    icon?: string;
    label?: string;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<Toggle.Root
  bind:pressed={() => pressed, onPressedChange}
  {disabled}
  aria-label={label}
  class="{chipTone[tone]} {className}"
>
  {#if icon}
    <Icon {icon} size="1.15em" class="shrink-0" aria-hidden="true" />
  {/if}
  {@render children?.()}
</Toggle.Root>
