<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { isKeybind, keyLabel } from '$lib/keybinds';
  import type { MaskComponent, MaskLayer } from '$lib/types/edits';
  import { lensWarpFromEdits, maskUvToSceneUv, type LensWarpParams } from '$lib/utils/lensWarp';
  import {
    displayUvToMaskUv,
    geometryTransformFrom,
    type GeometryTransform
  } from '$lib/utils/geomTransform';
  import { imageRect } from '$lib/utils/imageRect.svelte';
  import { mergeProps } from '$lib/utils/mergeProps';
  import { Tooltip } from '@immich/ui';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  const rect = imageRect(() => img);
  let boxStart = $state<[number, number] | null>(null);
  let boxNow = $state<[number, number] | null>(null);

  const MIN_DRAG_PX = 6;

  function onKeyDown(e: KeyboardEvent): void {
    if (!isKeybind(e, 'maskCancelDraw') || !editor.clickTool.box) return;
    e.preventDefault();
    boxStart = null;
    boxNow = null;
    editor.clickTool = { active: false, negative: false, box: false, layerId: null, mode: 'add' };
  }

  $effect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const geomT = $derived.by<GeometryTransform>(() =>
    geometryTransformFrom(editor.edits.geometry, editor.meta)
  );

  const lensP = $derived.by<LensWarpParams>(() =>
    lensWarpFromEdits(editor.lensView, editor.meta?.source_w ?? 1, editor.meta?.source_h ?? 1)
  );

  const active = $derived<MaskLayer | null>(
    editor.activeLayerId
      ? (editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null)
      : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? (active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null)
      : null
  );
  const points = $derived(
    activeComp?.generated?.kind === 'click' ? (activeComp.generated.points ?? []) : []
  );
  const show = $derived(
    editor.clickTool.active && editor.maskPreviewLayerId === null && rect.w > 0 && rect.h > 0
  );
  const dragRect = $derived(
    boxStart && boxNow
      ? {
          left: Math.min(boxStart[0], boxNow[0]),
          top: Math.min(boxStart[1], boxNow[1]),
          width: Math.abs(boxNow[0] - boxStart[0]),
          height: Math.abs(boxNow[1] - boxStart[1])
        }
      : null
  );

  function sceneUvFromDisplay(du: number, dv: number): [number, number] {
    return maskUvToSceneUv(lensP, displayUvToMaskUv(geomT, [du, dv]));
  }

  function sceneUvAt(e: PointerEvent): [number, number] {
    const bounds = (e.currentTarget as Element).getBoundingClientRect();
    const du = (e.clientX - bounds.left) / Math.max(1, bounds.width);
    const dv = (e.clientY - bounds.top) / Math.max(1, bounds.height);
    return sceneUvFromDisplay(du, dv);
  }

  function localPx(e: PointerEvent): [number, number] {
    const bounds = (e.currentTarget as Element).getBoundingClientRect();
    return [e.clientX - bounds.left, e.clientY - bounds.top];
  }

  function markerPos(x: number, y: number): { left: number; top: number } | null {
    const steps = 24;
    let bestLeft = 0;
    let bestTop = 0;
    let bestDist = Infinity;
    for (let i = 0; i <= steps; i++) {
      for (let j = 0; j <= steps; j++) {
        const du = i / steps;
        const dv = j / steps;
        const s = maskUvToSceneUv(lensP, displayUvToMaskUv(geomT, [du, dv]));
        const d = Math.hypot(s[0] - x, s[1] - y);
        if (d < bestDist) {
          bestDist = d;
          bestLeft = du;
          bestTop = dv;
        }
      }
    }
    if (bestDist > 0.08) return null;
    return { left: bestLeft * rect.w, top: bestTop * rect.h };
  }

  async function onPointerDown(e: PointerEvent): Promise<void> {
    if (editor.maskGenerating) return;
    e.preventDefault();
    if (editor.clickTool.box) {
      e.stopPropagation();
      boxStart = localPx(e);
      boxNow = boxStart;
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      return;
    }
    const [x, y] = sceneUvAt(e);
    if (x < 0 || y < 0 || x > 1 || y > 1) return;
    const positive = !(e.button === 2 || e.shiftKey || editor.clickTool.negative);
    await editor.addClickPoint(x, y, positive);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!boxStart) return;
    e.stopPropagation();
    boxNow = localPx(e);
  }

  async function onPointerUp(e: PointerEvent): Promise<void> {
    if (!boxStart || !boxNow) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    const [ax, ay] = boxStart;
    const [bx, by] = boxNow;
    boxStart = null;
    boxNow = null;
    if (Math.abs(bx - ax) < MIN_DRAG_PX || Math.abs(by - ay) < MIN_DRAG_PX) return;
    const corners: [number, number][] = [
      [ax, ay],
      [bx, ay],
      [ax, by],
      [bx, by]
    ];
    const scene = corners.map(([px, py]) =>
      sceneUvFromDisplay(px / Math.max(1, rect.w), py / Math.max(1, rect.h))
    );
    const xs = scene.map((s) => s[0]);
    const ys = scene.map((s) => s[1]);
    const bbox = {
      x0: Math.max(0, Math.min(...xs)),
      y0: Math.max(0, Math.min(...ys)),
      x1: Math.min(1, Math.max(...xs)),
      y1: Math.min(1, Math.max(...ys))
    };
    if (bbox.x1 - bbox.x0 < 0.005 || bbox.y1 - bbox.y0 < 0.005) return;
    await editor.addClickBox(bbox);
  }
</script>

{#if show}
  <div
    class="absolute"
    role="button"
    tabindex="-1"
    aria-label={editor.clickTool.box ? 'Drag a box around the subject' : 'Click to refine the mask'}
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none; cursor: {editor.maskGenerating
      ? 'wait'
      : 'crosshair'};"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#if dragRect}
      <div
        class="pointer-events-none absolute border-2 border-primary bg-primary/15"
        style="left: {dragRect.left}px; top: {dragRect.top}px; width: {dragRect.width}px; height: {dragRect.height}px;"
      ></div>
    {/if}
    {#if editor.clickTool.box && !dragRect}
      <div
        class="pointer-events-none absolute left-1/2 top-3 -translate-x-1/2 rounded bg-black/70 px-2.5 py-1 text-[11px] text-white shadow"
      >
        Drag a box around the subject · {keyLabel('Escape')} cancels
      </div>
    {/if}
    {#each points as p, i (i)}
      {@const pos = markerPos(p.x, p.y)}
      {#if pos}
        <Tooltip text="Remove this point">
          {#snippet child({ props })}
            <button
              type="button"
              class="absolute h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow transition hover:scale-125 hover:ring-2 hover:ring-white/60 disabled:cursor-wait"
              class:bg-emerald-500={p.positive}
              class:bg-red-500={!p.positive}
              style="left: {pos.left}px; top: {pos.top}px;"
              aria-label="Remove this point"
              disabled={editor.maskGenerating}
              {...mergeProps(props, {
                onpointerdown: (e: PointerEvent) => {
                  e.stopPropagation();
                  e.preventDefault();
                  void editor.removeClickPoint(i);
                }
              })}
            ></button>
          {/snippet}
        </Tooltip>
      {/if}
    {/each}
  </div>
{/if}
