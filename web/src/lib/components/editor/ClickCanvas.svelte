<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { isKeybind } from '#lib/shortcuts/match.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import type { MaskComponent, MaskLayer } from '#lib/types/edits.ts';
  import { lensWarpFromEdits, maskUvToSceneUv, type LensWarpParams } from '#lib/utils/lensWarp.ts';
  import {
    displayUvToMaskUv,
    geometryTransformFrom,
    type GeometryTransform
  } from '#lib/utils/geomTransform.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import { Icon, Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint, normalizedPoint } from '#lib/gestures/point.ts';
  import { mdiMinus, mdiPlus } from '@mdi/js';

  let {
    img
  }: {
    img: PreviewSurface | null;
  } = $props();

  const rect = imageRect(() => img);
  let boxStart = $state<[number, number] | null>(null);
  let boxNow = $state<[number, number] | null>(null);
  let hover = $state<[number, number] | null>(null);
  let inverting = $state(false);

  const MIN_DRAG_PX = 6;

  function onKeyDown(e: KeyboardEvent): void {
    inverting = held('exclude', e);
    if (!isKeybind(e, 'maskCancelDraw')) return;
    boxStart = null;
    boxNow = null;
  }

  function onKeyUp(e: KeyboardEvent): void {
    inverting = held('exclude', e);
  }

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
  const show = $derived(editor.clickTool.active && rect.w > 0 && rect.h > 0);
  const refining = $derived(activeComp?.kind.kind === 'brush');
  const including = $derived(!refining || editor.clickTool.negative === inverting);
  const tag = $derived(
    !refining
      ? 'Include'
      : `${including ? 'Include' : 'Exclude'} · ${keyLabel('Shift')} or right-click ${including ? 'excludes' : 'includes'}`
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
    const p = normalizedPoint(e, e.currentTarget as Element);
    return sceneUvFromDisplay(p.x, p.y);
  }

  function localPx(e: PointerEvent): [number, number] {
    const p = elementPoint(e, e.currentTarget as Element);
    return [p.x, p.y];
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

  async function addPoint(e: PointerEvent): Promise<void> {
    const [x, y] = sceneUvAt(e);
    if (x < 0 || y < 0 || x > 1 || y > 1) return;
    const invert = e.button === 2 || held('exclude', e);
    const positive = editor.clickTool.negative === invert;
    await editor.addClickPoint(x, y, positive);
  }

  const boxDrag = pointerDrag<true>({
    hover(e) {
      inverting = held('exclude', e);
      hover = e.pointerType === 'mouse' ? localPx(e) : null;
    },
    start(e) {
      e.stopPropagation();
      boxStart = localPx(e);
      boxNow = boxStart;
      return true;
    },
    move(e) {
      if (!boxStart) return;
      e.stopPropagation();
      boxNow = localPx(e);
    },
    end(e) {
      if (!boxStart || !boxNow) return;
      e.stopPropagation();
      void addBox(boxStart, boxNow);
    }
  });

  function onPointerDown(e: PointerEvent): void {
    if (editor.maskGenerating) return;
    e.preventDefault();
    if (editor.clickTool.box) boxDrag.begin(e);
    else void addPoint(e);
  }

  async function addBox(from: [number, number], to: [number, number]): Promise<void> {
    const [ax, ay] = from;
    const [bx, by] = to;
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

<svelte:window onkeydown={onKeyDown} onkeyup={onKeyUp} />

{#if show}
  <div
    class="absolute"
    role="button"
    tabindex="-1"
    aria-label={editor.clickTool.box ? 'Drag a box around the subject' : 'Click to select'}
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none; cursor: {editor.maskGenerating
      ? 'wait'
      : 'crosshair'};"
    onpointerdown={onPointerDown}
    {...boxDrag.follow}
    onpointerleave={() => (hover = null)}
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#if dragRect}
      <div
        class="pointer-events-none absolute border-2 border-image-accent bg-image-accent/15"
        style="left: {dragRect.left}px; top: {dragRect.top}px; width: {dragRect.width}px; height: {dragRect.height}px;"
      ></div>
    {/if}
    {#if hover && !editor.clickTool.box && !editor.maskGenerating}
      <div
        data-testid="click-cursor-tag"
        class="pointer-events-none absolute rounded bg-black/70 px-2 py-0.5 text-[11px] whitespace-nowrap text-white shadow"
        style="left: {hover[0] + 14}px; top: {hover[1] + 14}px;"
      >
        {tag}
      </div>
    {/if}
    {#each points as p, i (i)}
      {@const pos = markerPos(p.x, p.y)}
      {#if pos}
        <Tooltip text="Remove this point">
          {#snippet child({ props })}
            <button
              type="button"
              class="group absolute grid size-3 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full disabled:cursor-wait touch:size-11"
              data-testid="click-dot"
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
            >
              <span
                class="grid size-3 place-items-center rounded-full border-2 border-white shadow transition group-hover:scale-125 group-hover:ring-2 group-hover:ring-white/60 touch:size-6"
                class:bg-emerald-500={p.positive}
                class:bg-red-500={!p.positive}
              >
                <Icon
                  icon={p.positive ? mdiPlus : mdiMinus}
                  size="14px"
                  class="hidden text-white touch:block"
                  aria-hidden="true"
                />
              </span>
            </button>
          {/snippet}
        </Tooltip>
      {/if}
    {/each}
  </div>
{/if}
