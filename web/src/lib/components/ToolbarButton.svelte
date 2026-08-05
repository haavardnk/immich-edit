<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import type { Snippet } from 'svelte';

  let {
    path,
    title,
    ariaLabel,
    size = 20,
    active = false,
    disabled = false,
    pressed,
    variant = 'icon',
    label,
    onclick,
    onpointerdown,
    onpointerup,
    onpointerleave,
    children
  }: {
    path?: string;
    title?: string;
    ariaLabel?: string;
    size?: number;
    active?: boolean;
    disabled?: boolean;
    pressed?: boolean;
    variant?: 'icon' | 'text';
    label?: string;
    onclick?: (e: MouseEvent) => void;
    onpointerdown?: (e: PointerEvent) => void;
    onpointerup?: (e: PointerEvent) => void;
    onpointerleave?: (e: PointerEvent) => void;
    children?: Snippet;
  } = $props();

  const stateClass = $derived(
    disabled
      ? 'text-immich-dark-fg/25 cursor-not-allowed'
      : active
        ? 'text-immich-dark-primary hover:bg-white/10'
        : 'text-immich-dark-fg/70 hover:text-immich-dark-fg hover:bg-white/10'
  );
  const shapeClass = $derived(
    variant === 'text' ? 'h-7 min-w-13 px-2 font-mono text-xs' : 'size-7'
  );
</script>

<button
  type="button"
  class="inline-flex items-center justify-center rounded transition-colors {shapeClass} {stateClass}"
  {title}
  aria-label={ariaLabel ?? title}
  aria-pressed={pressed}
  {disabled}
  {onclick}
  {onpointerdown}
  {onpointerup}
  {onpointerleave}
>
  {#if children}
    {@render children()}
  {:else if variant === 'text'}
    {label}
  {:else if path}
    <Icon {path} {size} />
  {/if}
</button>
