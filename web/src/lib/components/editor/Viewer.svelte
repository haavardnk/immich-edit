<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';

  let container = $state<HTMLDivElement | null>(null);
  let dragging = $state(false);
  let lastX = 0;
  let lastY = 0;

  function onPointerDown(e: PointerEvent): void {
    if (ui.zoom <= 100) return;
    dragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!dragging) return;
    ui.panX += e.clientX - lastX;
    ui.panY += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
  }

  function onPointerUp(): void {
    dragging = false;
  }

  function onWheel(e: WheelEvent): void {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -10 : 10;
    ui.setZoom(ui.zoom + delta);
  }

  $effect(() => {
    editor.onZoomChange(ui.zoom);
  });
</script>

<div
  bind:this={container}
  role="application"
  class="flex-1 min-h-0 flex items-center justify-center bg-black/40 relative overflow-hidden"
  class:cursor-grab={ui.zoom > 100 && !dragging}
  class:cursor-grabbing={dragging}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
>
  {#if editor.previewUrl}
    <img
      src={editor.previewUrl}
      alt={editor.asset?.originalFileName ?? ''}
      class="max-w-full max-h-full object-contain shadow-2xl rounded select-none"
      style="transform: scale({ui.zoom / 100}) translate({ui.panX / (ui.zoom / 100)}px, {ui.panY / (ui.zoom / 100)}px); transform-origin: center; image-orientation: none;"
      draggable="false"
    />
  {:else if editor.error}
    <div class="text-red-400 text-sm">{editor.error}</div>
  {:else}
    <div class="flex gap-1">
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 0ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 150ms"></div>
      <div class="w-2 h-2 rounded-full bg-immich-dark-primary/50 animate-bounce" style="animation-delay: 300ms"></div>
    </div>
  {/if}
</div>
