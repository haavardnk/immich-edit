<script lang="ts">
  import { sourceToStage, stageToOutput, type StageFrame } from '#lib/editor/geometryStage.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import {
    cornerOffsetsFor,
    unitSquareCorners,
    type PerspectiveEdits
  } from '#lib/utils/perspective.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { HANDLE_DOT } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';

  interface Props {
    frame: StageFrame;
    perspective: PerspectiveEdits;
  }

  let { frame, perspective }: Props = $props();

  let root = $state<HTMLDivElement | null>(null);

  const hit = $derived(handleHit());
  const handles = $derived(unitSquareCorners().map((c) => sourceToStage(frame, c)));
  const quadPoints = $derived(handles.map(([x, y]) => `${x},${y}`).join(' '));

  const corner = pointerDrag<number, number>({
    start(e, index) {
      e.preventDefault();
      e.stopPropagation();
      return index;
    },
    move(e, index) {
      e.stopPropagation();
      if (!root) return;
      const at = elementPoint(e, root);
      editor.updateGeometryDraftPerspective({
        corners: cornerOffsetsFor(perspective, index, stageToOutput(frame, [at.x, at.y]))
      });
    },
    end(e) {
      e.stopPropagation();
      void editor.commitGeometry('Perspective');
    }
  });
</script>

<div bind:this={root} class="pointer-events-none absolute inset-0">
  <svg class="absolute inset-0" width={frame.bboxW} height={frame.bboxH} aria-hidden="true">
    <polygon
      points={quadPoints}
      fill="none"
      stroke="var(--color-image-accent)"
      stroke-width="1"
      stroke-dasharray="4 3"
    />
  </svg>
  {#each handles as [x, y], i (i)}
    <button
      class="pointer-events-auto absolute cursor-grab active:cursor-grabbing"
      style="width: {hit * 2}px; height: {hit * 2}px; left: {x - hit}px; top: {y - hit}px;"
      {...corner.on(i)}
      aria-label="perspective corner {i + 1}"
    >
      <span
        class="pointer-events-none absolute rounded-full border border-black/60 bg-image-accent"
        style="inset: {hit - HANDLE_DOT.corner}px;"
      ></span>
    </button>
  {/each}
</div>
