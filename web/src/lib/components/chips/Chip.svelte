<script lang="ts">
  import { chip } from '#lib/components/chips/chipStyles.ts';
  import { twMerge } from '#lib/theme/tailwindMerge.ts';
  import { Icon } from '@immich/ui';
  import { Toggle } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let {
    pressed,
    onPressedChange,
    icon,
    label,
    title,
    disabled = false,
    class: className = '',
    children
  }: {
    pressed: boolean;
    onPressedChange: (pressed: boolean) => void;
    icon?: string;
    label?: string;
    title?: string;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<Toggle.Root
  bind:pressed={() => pressed, onPressedChange}
  {disabled}
  aria-label={label}
  {title}
  class={twMerge(chip, className)}
>
  {#if icon}
    <Icon {icon} size="1.15em" class="shrink-0" aria-hidden="true" />
  {/if}
  {@render children?.()}
</Toggle.Root>
