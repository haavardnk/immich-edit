<svelte:options namespace="svg" />

<script lang="ts">
  import type { MaskHandleProps } from '#lib/types/maskHandle.ts';

  let {
    cx,
    cy,
    r,
    hit,
    label,
    cursor,
    fill,
    fillOpacity,
    stroke,
    strokeWidth,
    onpointerdown,
    ondblclick
  }: MaskHandleProps = $props();
</script>

<circle
  {cx}
  {cy}
  r={hit}
  fill="transparent"
  style="cursor: {cursor}; pointer-events: all;"
  role="button"
  aria-label={label}
  tabindex="-1"
  {onpointerdown}
  {ondblclick}
/>
<circle
  {cx}
  {cy}
  {r}
  {fill}
  fill-opacity={fillOpacity}
  {stroke}
  stroke-width={strokeWidth}
  style="pointer-events: none;"
/>
