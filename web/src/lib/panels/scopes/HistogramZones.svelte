<script lang="ts">
  import { beginParamDrag, type ParamDrag } from '#lib/editor/paramDrag.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { clampedPoint } from '#lib/gestures/point.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { zoneAt, zoneValue, type HistogramZone } from './histogramZones';

  type Dragging = { session: ParamDrag; zone: HistogramZone; x: number; width: number };

  let hovered = $state<HistogramZone | null>(null);
  let dragging = $state.raw<Dragging | null>(null);

  const zone = $derived(dragging?.zone ?? hovered);
  const value = $derived(zone ? zone.param.format(zone.param.get(editor.edits, editor.isRaw)) : '');
  const labelStyle = $derived(
    !zone
      ? ''
      : zone.from === 0
        ? 'left: 2px;'
        : zone.to === 1
          ? 'right: 2px;'
          : `left: ${((zone.from + zone.to) / 2) * 100}%; transform: translateX(-50%);`
  );

  function zoneOf(e: PointerEvent): HistogramZone {
    return zoneAt(clampedPoint(e, e.currentTarget as Element).x);
  }

  const drag = pointerDrag<true>({
    start(e) {
      if (e.pointerType === 'touch' || e.button !== 0 || !editor.assetId) return null;
      e.preventDefault();
      const picked = zoneOf(e);
      dragging = {
        session: beginParamDrag(picked.param),
        zone: picked,
        x: e.clientX,
        width: (e.currentTarget as Element).getBoundingClientRect().width
      };
      return true;
    },
    hover(e) {
      if (e.pointerType !== 'touch') hovered = zoneOf(e);
    },
    move(e) {
      if (!dragging) return;
      const { session, x, width } = dragging;
      session.set(zoneValue(session.param, session.start, e.clientX - x, width));
    },
    end() {
      dragging?.session.end();
      dragging = null;
    }
  });
</script>

<div
  data-testid="histogram-zones"
  class="absolute inset-0 cursor-ew-resize"
  {...drag.on()}
  onpointerleave={() => (hovered = null)}
>
  {#if zone}
    <span
      class="pointer-events-none absolute inset-y-0 border-x border-primary/45 bg-primary/12"
      style:left="{zone.from * 100}%"
      style:width="{(zone.to - zone.from) * 100}%"
    ></span>
    <span
      data-testid="histogram-zone"
      class="pointer-events-none absolute bottom-3 rounded bg-editor-chrome/85 px-1.5 py-px text-[10px] font-medium whitespace-nowrap text-light-950 tabular-nums"
      style={labelStyle}
    >
      {zone.param.label}
      {value}
    </span>
  {/if}
</div>
