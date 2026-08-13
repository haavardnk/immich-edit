<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { isKeybind, isTypingTarget, keysFor } from '$lib/keybinds';
  import { type Vec2f } from '$lib/types/edits';
  import { displayUvToSceneUv, sceneUvToDisplayUv, viewTransform } from '$lib/utils/canvasCoords';
  import { clamp01 } from '$lib/utils/geom';
  import { draggedKind, type DragKind } from '$lib/utils/maskDrag';
  import { imageRect } from '$lib/utils/imageRect.svelte';
  import MaskLinearHandles from './MaskLinearHandles.svelte';
  import MaskRadialHandles from './MaskRadialHandles.svelte';
  import MaskPolygonHandles from './MaskPolygonHandles.svelte';
  import MaskPolygonDraft from './MaskPolygonDraft.svelte';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  const rect = imageRect(() => img);

  const view = $derived(viewTransform(editor.edits, editor.meta, editor.lensView));

  const active = $derived(
    editor.activeLayerId
      ? (editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null)
      : null
  );

  const showOverlay = $derived(
    editor.maskOverlayVisible &&
      !!active &&
      editor.maskPreviewLayerId === null &&
      rect.w > 0 &&
      rect.h > 0
  );
  const showColorPicker = $derived(
    !!editor.colorPicker?.ready && rect.w > 0 && rect.h > 0 && !!img
  );

  function toPx(scene: Vec2f): { x: number; y: number } {
    const d = sceneUvToDisplayUv(view, scene.x, scene.y);
    return { x: rect.x + d[0] * rect.w, y: rect.y + d[1] * rect.h };
  }

  function fromPx(px: number, py: number): Vec2f {
    const du = clamp01((px - rect.x) / Math.max(rect.w, 1));
    const dv = clamp01((py - rect.y) / Math.max(rect.h, 1));
    const s = displayUvToSceneUv(view, du, dv);
    return { x: s[0], y: s[1] };
  }

  let drag = $state<{ componentId: string; kind: DragKind } | null>(null);

  function startDrag(e: PointerEvent, componentId: string, kind: DragKind): void {
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    editor.setActiveMaskComponent(componentId);
    drag = { componentId, kind };
  }

  function selectOnly(e: PointerEvent, componentId: string): void {
    e.stopPropagation();
    editor.setActiveMaskComponent(componentId);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (isKeybind(e, 'maskCancelDraw') && editor.colorPicker) {
      e.preventDefault();
      editor.cancelColorPicker();
      return;
    }
    if (!isKeybind(e, 'maskDelete')) return;
    if (ui.editorTab !== 'masks' || isTypingTarget(e)) return;
    if (!active || !editor.activeMaskComponentId) return;
    e.preventDefault();
    void editor.removeMaskComponent(active.id, editor.activeMaskComponentId);
    toasts.push('info', `Shape deleted. Undo with ${keysFor('undo')}.`);
  }

  $effect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  function onPointerMove(e: PointerEvent): void {
    if (!drag || !active) return;
    const current = drag;
    const comp = active.components.find((c) => c.id === current.componentId);
    if (!comp) return;
    const r = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const n = fromPx(e.clientX - r.left, e.clientY - r.top);
    const next = draggedKind(comp.kind, current.kind, n);
    if (next) editor.updateMaskComponentKind(active.id, comp.id, next, true);
  }

  function onPointerUp(e: PointerEvent): void {
    if (!drag) return;
    drag = null;
    (e.currentTarget as Element).releasePointerCapture?.(e.pointerId);
    void editor.commitMasks();
  }

  function setPolygonPoints(componentId: string, points: Vec2f[]): void {
    if (!active) return;
    const comp = active.components.find((c) => c.id === componentId);
    if (!comp || comp.kind.kind !== 'polygon') return;
    editor.updateMaskComponentKind(active.id, comp.id, { ...comp.kind, points }, false);
    void editor.commitMasks();
  }

  function sampleColor(e: PointerEvent): void {
    e.preventDefault();
    e.stopPropagation();
    if (!img || !editor.colorPicker?.ready || img.naturalWidth < 1 || img.naturalHeight < 1) return;
    const bounds = img.getBoundingClientRect();
    const u = clamp01((e.clientX - bounds.left) / Math.max(bounds.width, 1));
    const v = clamp01((e.clientY - bounds.top) / Math.max(bounds.height, 1));
    const sx = Math.min(img.naturalWidth - 1, Math.floor(u * img.naturalWidth));
    const sy = Math.min(img.naturalHeight - 1, Math.floor(v * img.naturalHeight));
    const canvas = document.createElement('canvas');
    canvas.width = 1;
    canvas.height = 1;
    const context = canvas.getContext('2d', { willReadFrequently: true });
    if (!context) return;
    context.drawImage(img, sx, sy, 1, 1, 0, 0, 1, 1);
    const pixel = context.getImageData(0, 0, 1, 1).data;
    void editor.commitColorSample([pixel[0] / 255, pixel[1] / 255, pixel[2] / 255]);
  }

  const draft = $derived(editor.polygonDraft);
  const drafting = $derived(!!draft && rect.w > 0 && rect.h > 0);
  const activeCompId = $derived(editor.activeMaskComponentId);
</script>

{#if showColorPicker}
  <button
    type="button"
    class="absolute z-30 cursor-crosshair bg-transparent"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
    aria-label="Sample mask color"
    onpointerdown={sampleColor}
  ></button>
{/if}

{#if showOverlay && active}
  <svg
    class="absolute inset-0 pointer-events-none"
    width="100%"
    height="100%"
    role="presentation"
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
  >
    {#each active.components as comp (comp.id)}
      {#if comp.enabled && activeCompId === comp.id}
        {#if comp.kind.kind === 'linear'}
          <MaskLinearHandles
            {comp}
            kind={comp.kind}
            color={active.color}
            {rect}
            {toPx}
            {fromPx}
            onSelect={(e) => selectOnly(e, comp.id)}
            onDrag={(e, kind) => startDrag(e, comp.id, kind)}
          />
        {:else if comp.kind.kind === 'radial'}
          <MaskRadialHandles
            {comp}
            kind={comp.kind}
            color={active.color}
            {rect}
            {toPx}
            onSelect={(e) => selectOnly(e, comp.id)}
            onDrag={(e, kind) => startDrag(e, comp.id, kind)}
          />
        {:else if comp.kind.kind === 'polygon'}
          <MaskPolygonHandles
            kind={comp.kind}
            color={active.color}
            {toPx}
            {fromPx}
            onDrag={(e, kind) => startDrag(e, comp.id, kind)}
            onPoints={(points) => setPolygonPoints(comp.id, points)}
          />
        {/if}
      {/if}
    {/each}
  </svg>
{/if}

{#if drafting && draft}
  <MaskPolygonDraft {draft} {rect} {toPx} {fromPx} />
{/if}
