<script lang="ts">
  import { sourceToStage, stageToOutput, type StageFrame } from '$lib/editor/geometryStage';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    cornerOffsetsFor,
    unitSquareCorners,
    type PerspectiveEdits
  } from '$lib/utils/perspective';

  interface Props {
    frame: StageFrame;
    perspective: PerspectiveEdits;
  }

  let { frame, perspective }: Props = $props();

  let root = $state<HTMLDivElement | null>(null);
  let dragging = $state<number | null>(null);

  const handles = $derived(unitSquareCorners().map((c) => sourceToStage(frame, c)));
  const quadPoints = $derived(handles.map(([x, y]) => `${x},${y}`).join(' '));

  function startDrag(e: PointerEvent, index: number): void {
    e.preventDefault();
    e.stopPropagation();
    dragging = index;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onMove(e: PointerEvent): void {
    if (dragging === null || !root) return;
    e.stopPropagation();
    const rect = root.getBoundingClientRect();
    const uv = stageToOutput(frame, [e.clientX - rect.left, e.clientY - rect.top]);
    editor.updateGeometryDraftPerspective({
      corners: cornerOffsetsFor(perspective, dragging, uv)
    });
  }

  function onUp(e: PointerEvent): void {
    e.stopPropagation();
    const dragged = dragging !== null;
    dragging = null;
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    if (dragged) void editor.commitGeometry('Perspective');
  }
</script>

<div bind:this={root} class="pointer-events-none absolute inset-0">
  <svg class="absolute inset-0" width={frame.bboxW} height={frame.bboxH} aria-hidden="true">
    <polygon
      points={quadPoints}
      fill="none"
      stroke="var(--color-primary)"
      stroke-width="1"
      stroke-dasharray="4 3"
    />
  </svg>
  {#each handles as [x, y], i (i)}
    <button
      class="pointer-events-auto absolute cursor-grab rounded-full border border-black/60 bg-primary active:cursor-grabbing"
      style="width: 14px; height: 14px; left: {x - 7}px; top: {y - 7}px;"
      onpointerdown={(e) => startDrag(e, i)}
      onpointermove={onMove}
      onpointerup={onUp}
      onpointercancel={onUp}
      aria-label="perspective corner {i + 1}"
    ></button>
  {/each}
</div>
