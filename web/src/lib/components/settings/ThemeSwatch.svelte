<script lang="ts">
  import type { Palette } from '#lib/theme/types.ts';

  let {
    palette,
    accent,
    size = 'tile'
  }: { palette: Palette; accent: string; size?: 'tile' | 'chip' } = $props();
</script>

<span
  aria-hidden="true"
  class="relative block shrink-0 overflow-hidden ring-1 ring-hairline {size === 'tile'
    ? 'h-16 w-full rounded-lg transition-shadow group-hover:ring-light-950/35 group-focus-visible:ring-2 group-focus-visible:ring-primary group-aria-checked:ring-2 group-aria-checked:ring-primary'
    : 'h-6 w-9 rounded-md touch:h-7 touch:w-11'}"
  style:background-color={palette.base}
>
  {#if size === 'tile'}
    <span class="absolute inset-x-0 top-0 h-3" style:background-color={palette.mantle}></span>
    <span
      class="absolute top-6 left-2.5 h-1.5 w-14 rounded-full"
      style:background-color={palette.text}
    ></span>
    <span
      class="absolute top-9.5 left-2.5 h-1.5 w-9 rounded-full"
      style:background-color={palette.subtext}
    ></span>
    <span class="absolute right-2.5 bottom-2.5 size-4 rounded-full" style:background-color={accent}
    ></span>
  {:else}
    <span class="absolute right-1 bottom-1 size-2 rounded-full" style:background-color={accent}
    ></span>
  {/if}
</span>
