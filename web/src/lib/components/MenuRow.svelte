<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    icon,
    label,
    keys,
    pressed,
    class: className = '',
    children,
    ...rest
  }: {
    icon?: string;
    label?: string;
    keys?: string;
    pressed?: boolean;
    class?: string;
    children?: Snippet;
  } & Omit<HTMLButtonAttributes, 'children' | 'color' | 'size' | 'type' | 'class'> = $props();

  const showKeys = $derived(Boolean(keys) && (layout.mode === 'desktop' || layout.hasKeyboard));
</script>

<Button
  {...rest}
  type="button"
  size="tiny"
  variant="ghost"
  color={pressed ? 'primary' : 'secondary'}
  class="h-7 w-full justify-start gap-2.5 rounded-none px-3 text-left text-xs font-normal touch:h-13 touch:gap-3.5 touch:border-b touch:border-hairline touch:px-4 touch:text-base touch:last:border-b-0 {className}"
  data-menu-row
  aria-pressed={pressed}
>
  {#if icon}
    <Icon {icon} size="14px" class="shrink-0 opacity-70 touch:size-5" aria-hidden="true" />
  {/if}
  {#if label}
    <span class="min-w-0 flex-1 truncate">{label}</span>
  {/if}
  {@render children?.()}
  {#if showKeys}
    <span class="shrink-0 font-mono text-[10px] text-dark/65" aria-hidden="true">{keys}</span>
  {/if}
</Button>
