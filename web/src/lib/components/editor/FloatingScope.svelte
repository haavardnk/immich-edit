<script lang="ts">
  import { DRAG_SLOP, nearestCorner, yieldsTo, type Corner } from '#lib/editor/floatingScope.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import type { Point } from '#lib/gestures/point.ts';
  import ScopePlot from '#lib/panels/scopes/ScopePlot.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { scopes, type ScopeMode } from '#lib/stores/scopes.svelte.ts';
  import { sliderDrag } from '#lib/stores/sliderDrag.svelte.ts';

  let { yieldToSliders = false }: { yieldToSliders?: boolean } = $props();

  const LABELS: Record<ScopeMode, string> = {
    histogram: 'Histogram',
    waveform: 'Waveform',
    parade: 'Parade',
    vectorscope: 'Vectorscope'
  };

  const POSITIONS: Record<Corner, string> = {
    tl: 'top-3.5 left-3.5 [[data-arrangement=side]_&]:top-15',
    tr: 'top-3.5 right-3.5 [[data-arrangement=side]_&]:top-15',
    bl: 'bottom-3.5 left-3.5',
    br: 'bottom-3.5 right-3.5'
  };

  type Grab = Point & { dragging: boolean };

  let element = $state<HTMLButtonElement | null>(null);
  let offset = $state<Point | null>(null);
  let plotHeight = $state(0);
  let dragged = false;

  const hidden = $derived(yieldToSliders && yieldsTo(sliderDrag.setting));
  const label = $derived(LABELS[scopes.mode]);

  function settle(): void {
    const stage = element?.offsetParent;
    if (!element || !stage) return;
    const box = element.getBoundingClientRect();
    const bounds = stage.getBoundingClientRect();
    const centre = {
      x: box.left + box.width / 2 - bounds.left,
      y: box.top + box.height / 2 - bounds.top
    };
    scopes.setCorner(nearestCorner(centre, { w: bounds.width, h: bounds.height }));
  }

  const drag = pointerDrag<Grab>({
    start(e) {
      if (e.button !== 0) return null;
      dragged = false;
      return { x: e.clientX, y: e.clientY, dragging: false };
    },
    move(e, grab) {
      const x = e.clientX - grab.x;
      const y = e.clientY - grab.y;
      if (!grab.dragging && Math.hypot(x, y) <= DRAG_SLOP) return;
      grab.dragging = true;
      dragged = true;
      offset = { x, y };
    },
    end(_e, grab) {
      if (grab.dragging) settle();
      offset = null;
    },
    cancel() {
      offset = null;
    }
  });

  const handlers = drag.on();

  function onpointerdown(e: PointerEvent): void {
    e.stopPropagation();
    handlers.onpointerdown(e);
  }

  function onclick(): void {
    if (dragged) {
      dragged = false;
      return;
    }
    scopes.cycleFloating();
    if (scopes.needsRender) editor.refreshScopes();
  }
</script>

{#if scopes.floating && !editor.geometrySession}
  <button
    bind:this={element}
    type="button"
    data-testid="floating-scope"
    data-corner={scopes.corner}
    aria-label="{label} overlay"
    class="absolute z-10 h-30 w-70 touch-none overflow-hidden rounded-lg bg-black/60 text-left text-white shadow-lg ring-1 ring-white/14 backdrop-blur-sm transition-opacity select-none touch:h-19 touch:w-34 touch:rounded-[10px] touch:bg-black/45 {offset
      ? 'cursor-grabbing'
      : 'cursor-grab'} {POSITIONS[scopes.corner]} {hidden ? 'pointer-events-none opacity-0' : ''}"
    style:translate={offset ? `${offset.x}px ${offset.y}px` : undefined}
    {...handlers}
    {onpointerdown}
    ondblclick={(e) => e.stopPropagation()}
    {onclick}
  >
    <span
      class="pointer-events-none absolute inset-x-0 top-5 bottom-0"
      bind:clientHeight={plotHeight}
    >
      {#if plotHeight > 0}
        <ScopePlot height={plotHeight} mini />
      {/if}
    </span>
    <span class="pointer-events-none absolute top-1 left-2 text-[10.5px] text-white/90"
      >{label}</span
    >
    <span class="pointer-events-none absolute top-2.25 right-2 flex gap-1" aria-hidden="true">
      <span
        class="size-1.25 rounded-full {scopes.mode === 'histogram' ? 'bg-white/95' : 'bg-white/35'}"
      ></span>
      <span
        class="size-1.25 rounded-full {scopes.mode === 'histogram' ? 'bg-white/35' : 'bg-white/95'}"
      ></span>
    </span>
  </button>
{/if}
