<svelte:options namespace="svg" />

<script lang="ts">
  import { HANDLE_HIT } from '#lib/editor/handles.ts';

  let {
    cx,
    cy,
    r,
    hit = HANDLE_HIT,
    label,
    cursor,
    fill,
    fillOpacity,
    stroke,
    strokeWidth,
    onpointerdown,
    ondblclick
  }: {
    cx: number;
    cy: number;
    r: number;
    hit?: number;
    label: string;
    cursor: string;
    fill: string;
    fillOpacity?: number;
    stroke: string;
    strokeWidth: number;
    onpointerdown: (e: PointerEvent) => void;
    ondblclick?: (e: MouseEvent) => void;
  } = $props();
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
