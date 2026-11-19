<script lang="ts">
  import { Icon } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    label,
    hint = null,
    anchor,
    icon,
    stack = false,
    muted = false,
    badge,
    detail,
    children
  }: {
    label: string;
    hint?: string | null;
    anchor?: string;
    icon?: string;
    stack?: boolean;
    muted?: boolean;
    badge?: Snippet;
    detail?: Snippet;
    children?: Snippet;
  } = $props();
</script>

<div
  data-setting-anchor={anchor}
  class="relative flex min-h-14 items-center gap-4 px-4 py-2.5 touch:min-h-13 {stack
    ? 'touch:flex-col touch:items-stretch touch:gap-2.5'
    : ''}"
>
  {#if icon}
    <Icon {icon} size="20px" class="shrink-0 text-dark/80" aria-hidden="true" />
  {/if}
  <div class="min-w-0 flex-1">
    <div
      class="flex min-w-0 items-center gap-2 text-label touch:text-body {muted
        ? 'text-dark/80'
        : 'text-light-950'}"
    >
      <span class="truncate">{label}</span>
      {@render badge?.()}
    </div>
    {#if hint}
      <div class="mt-0.5 text-xs text-dark/80 touch:text-label">{hint}</div>
    {/if}
    {#if detail}
      <div class="mt-0.5 truncate text-xs text-dark/80 touch:text-label">{@render detail()}</div>
    {/if}
  </div>
  {@render children?.()}
</div>
