<script lang="ts">
  import PreviewImage from '#lib/components/editor/PreviewImage.svelte';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { clampedPoint } from '#lib/gestures/point.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import CompareLabel from './CompareLabel.svelte';
  import { paneLabels } from './compareText.ts';

  type Size = { w: number; h: number };

  let {
    surface = $bindable(null),
    frame,
    edge,
    onsize
  }: {
    surface?: PreviewSurface | null;
    frame: string;
    edge: string;
    onsize: (size: Size) => void;
  } = $props();

  let wrap = $state<HTMLDivElement | null>(null);
  const labels = $derived(paneLabels());

  const split = pointerDrag<true>({
    start(e) {
      move(e);
      e.stopPropagation();
      return true;
    },
    move: (e) => move(e)
  });

  function move(e: PointerEvent): void {
    if (!wrap) return;
    editor.compare.setSplitPos(clampedPoint(e, wrap).x);
  }

  function onkeydown(e: KeyboardEvent): void {
    const step = held('coarse', e) ? 0.1 : 0.02;
    if (e.key === 'ArrowLeft') editor.compare.setSplitPos(editor.compare.splitPos - step);
    else if (e.key === 'ArrowRight') editor.compare.setSplitPos(editor.compare.splitPos + step);
    else if (e.key === 'Home') editor.compare.setSplitPos(0);
    else if (e.key === 'End') editor.compare.setSplitPos(1);
    else return;
    e.preventDefault();
  }
</script>

<div
  bind:this={wrap}
  data-photo-layer
  data-compare="split"
  class="absolute overflow-hidden {edge}"
  style={frame}
>
  <img
    bind:this={surface}
    src={editor.compareUrl}
    alt={labels.left}
    data-photo-layer
    class="absolute inset-0 h-full w-full object-contain select-none"
    style="image-orientation: none;"
    draggable="false"
  />
  <PreviewImage
    url={editor.previewUrl}
    frame={editor.previewFrame}
    alt={editor.asset?.originalFileName ?? ''}
    class="absolute inset-0 h-full w-full object-contain select-none"
    style="clip-path: inset(0 0 0 {editor.compare.splitPos * 100}%);"
    {onsize}
  />
  <div
    class="pointer-events-none absolute top-0 bottom-0 w-0.5 bg-white/90 shadow-split"
    style="left: {editor.compare.splitPos * 100}%; transform: translateX(-50%);"
  ></div>
  <div
    role="slider"
    tabindex="0"
    aria-label="Before/after split"
    aria-valuenow={Math.round(editor.compare.splitPos * 100)}
    aria-valuemin="0"
    aria-valuemax="100"
    class="absolute top-1/2 flex h-7 w-7 -translate-x-1/2 -translate-y-1/2 cursor-ew-resize items-center justify-center rounded-full border-2 border-black/40 bg-white text-xs font-bold text-black/70 shadow-lg"
    style="left: {editor.compare.splitPos * 100}%;"
    {...split.on()}
    {onkeydown}
  >
    ↔
  </div>
  <CompareLabel text={labels.left} class="left-2.5" />
  <CompareLabel text={labels.right} class="right-2.5" />
</div>
