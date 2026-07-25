<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import type { MaskComponent, MaskLayer } from '$lib/types/edits';
  import {
    lensWarpFromEdits,
    maskUvToSceneUv,
    type LensWarpParams
  } from '$lib/utils/lensWarp';
  import {
    displayUvToMaskUv,
    type GeometryTransform,
    type RotateQuarter
  } from '$lib/utils/geomTransform';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  let rectX = $state(0);
  let rectY = $state(0);
  let rectW = $state(0);
  let rectH = $state(0);

  function recompute(): void {
    if (!img) return;
    const parent = img.parentElement;
    if (!parent) return;
    const p = parent.getBoundingClientRect();
    const r = img.getBoundingClientRect();
    rectX = r.left - p.left;
    rectY = r.top - p.top;
    rectW = r.width;
    rectH = r.height;
  }

  $effect(() => {
    if (!img) return;
    recompute();
    const ro = new ResizeObserver(recompute);
    ro.observe(img);
    if (img.parentElement) ro.observe(img.parentElement);
    img.addEventListener('load', recompute);
    window.addEventListener('resize', recompute);
    return () => {
      ro.disconnect();
      img.removeEventListener('load', recompute);
      window.removeEventListener('resize', recompute);
    };
  });

  $effect(() => {
    void ui.zoom;
    void ui.panX;
    void ui.panY;
    if (!img) return;
    const id = requestAnimationFrame(recompute);
    return () => cancelAnimationFrame(id);
  });

  const geomT = $derived.by<GeometryTransform>(() => {
    const g = editor.edits.geometry;
    const sw = editor.meta?.source_w ?? 1;
    const sh = editor.meta?.source_h ?? 1;
    const dw = editor.meta?.width ?? sw;
    const dh = editor.meta?.height ?? sh;
    return {
      inputW: sw,
      inputH: sh,
      rotateQuarter: g.rotate as RotateQuarter,
      flipH: g.flip_h,
      flipV: g.flip_v,
      angleDeg: g.rotate_angle,
      crop: g.crop ?? { x: 0, y: 0, w: 1, h: 1 },
      outputW: dw,
      outputH: dh
    };
  });

  const lensP = $derived.by<LensWarpParams>(() =>
    lensWarpFromEdits(editor.edits.lens, editor.meta?.source_w ?? 1, editor.meta?.source_h ?? 1)
  );

  const active = $derived<MaskLayer | null>(
    editor.activeLayerId
      ? editor.edits.masks.find((l) => l.id === editor.activeLayerId) ?? null
      : null
  );
  const activeComp = $derived<MaskComponent | null>(
    active && editor.activeMaskComponentId
      ? active.components.find((c) => c.id === editor.activeMaskComponentId) ?? null
      : null
  );
  const points = $derived(
    activeComp?.generated?.kind === 'click' ? (activeComp.generated.points ?? []) : []
  );
  const show = $derived(
    editor.clickTool.active && editor.maskPreviewLayerId === null && rectW > 0 && rectH > 0
  );

  function sceneUvAt(e: PointerEvent): [number, number] {
    const rect = (e.currentTarget as Element).getBoundingClientRect();
    const du = (e.clientX - rect.left) / Math.max(1, rect.width);
    const dv = (e.clientY - rect.top) / Math.max(1, rect.height);
    return maskUvToSceneUv(lensP, displayUvToMaskUv(geomT, [du, dv]));
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
    return { left: bestLeft * rectW, top: bestTop * rectH };
  }

  async function onPointerDown(e: PointerEvent): Promise<void> {
    if (editor.maskGenerating) return;
    e.preventDefault();
    const [x, y] = sceneUvAt(e);
    if (x < 0 || y < 0 || x > 1 || y > 1) return;
    const positive = !(e.button === 2 || e.shiftKey || editor.clickTool.negative);
    await editor.addClickPoint(x, y, positive);
  }
</script>

{#if show}
  <div
    class="absolute"
    role="button"
    tabindex="-1"
    aria-label="Click to refine the mask"
    style="left: {rectX}px; top: {rectY}px; width: {rectW}px; height: {rectH}px; touch-action: none; cursor: {editor.maskGenerating
      ? 'wait'
      : 'crosshair'};"
    onpointerdown={onPointerDown}
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#each points as p, i (i)}
      {@const pos = markerPos(p.x, p.y)}
      {#if pos}
        <span
          class="pointer-events-none absolute h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow"
          class:bg-emerald-500={p.positive}
          class:bg-red-500={!p.positive}
          style="left: {pos.left}px; top: {pos.top}px;"
        ></span>
      {/if}
    {/each}
  </div>
{/if}
