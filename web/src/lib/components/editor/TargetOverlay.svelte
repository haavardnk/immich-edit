<script lang="ts">
  import { neutraliseSection } from '#lib/editor/params/neutralise.ts';
  import {
    beginTargetDrag,
    selectTarget,
    type TargetAxis,
    type TargetDrag
  } from '#lib/editor/targetSession.ts';
  import { TARGET_TOOLS, targetAvailable, type TargetHit } from '#lib/editor/targetTools.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { clampedPoint } from '#lib/gestures/point.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { TargetReference } from '#lib/stores/editor/targetReference.svelte.ts';
  import { targetAdjust } from '#lib/stores/targetAdjust.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import type { Edits } from '#lib/types/edits.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { untrack } from 'svelte';
  import TargetCallout from './TargetCallout.svelte';
  import TouchMagnifier from './TouchMagnifier.svelte';

  type Spot = { u: number; v: number };
  type Adjusting = { session: TargetDrag; hit: TargetHit; startY: number };

  let { img }: { img: PreviewSurface | null } = $props();

  const rect = imageRect(() => img);
  const reference = new TargetReference();
  const def = $derived(
    targetAdjust.tool && targetAvailable(targetAdjust.tool, editor.edits)
      ? TARGET_TOOLS[targetAdjust.tool]
      : null
  );
  const active = $derived(!!def && rect.w > 0 && rect.h > 0 && !!img);
  let pointer = $state<Spot | null>(null);
  let touching = $state(false);
  let adjusting = $state.raw<Adjusting | null>(null);

  const hover = $derived.by((): TargetHit | null => {
    if (!def || !pointer) return null;
    const rgb = reference.sample(pointer.u, pointer.v);
    return rgb ? def.hit(rgb, editor.edits, curveEditor.channel) : null;
  });

  $effect(() => {
    const assetId = editor.assetId;
    if (!active || !def || !assetId) {
      reference.clear();
      return;
    }
    const edits = $state.snapshot(editor.edits) as Edits;
    reference.sync({ assetId, edits: neutraliseSection(edits, def.section, editor.isRaw) });
  });

  $effect(() => {
    if (!targetAdjust.tool) return;
    untrack(() => editor.cancelWbPicker());
  });

  $effect(() => {
    if (!editor.wbPicking && ui.editorTab === 'develop') return;
    untrack(() => targetAdjust.exit());
  });

  $effect(() => {
    const tool = targetAdjust.tool;
    if (!tool || targetAvailable(tool, editor.edits)) return;
    untrack(() => targetAdjust.exit());
  });

  $effect(() => () => targetAdjust.exit());

  $effect(() => {
    if (active) return;
    untrack(() => {
      pointer = null;
      touching = false;
    });
  });

  function spotAt(e: PointerEvent): Spot {
    const at = clampedPoint(e, e.currentTarget as Element);
    return { u: at.x, v: at.y };
  }

  function axisOf(e: PointerEvent): TargetAxis {
    if (held('hueAxis', e)) return 'hue';
    return held('lightnessAxis', e) ? 'lightness' : 'main';
  }

  function finishAdjust(): void {
    adjusting?.session.end();
    adjusting = null;
  }

  const press = pointerDrag<'pick' | 'adjust'>({
    start(e) {
      e.preventDefault();
      e.stopPropagation();
      if (e.button !== 0 || !def) return null;
      touching = e.pointerType === 'touch';
      pointer = spotAt(e);
      if (touching) return 'pick';
      const hit = hover;
      if (!hit) return null;
      adjusting = { session: beginTargetDrag(def, hit, axisOf(e)), hit, startY: e.clientY };
      return 'adjust';
    },
    hover(e) {
      if (e.pointerType !== 'touch') pointer = spotAt(e);
    },
    move(e, mode) {
      if (mode === 'pick') pointer = spotAt(e);
      else adjusting?.session.move(e.clientY - adjusting.startY);
    },
    end(e, mode) {
      e.stopPropagation();
      if (mode === 'adjust') {
        finishAdjust();
        return;
      }
      const hit = hover;
      pointer = null;
      touching = false;
      if (!hit || !def) return;
      selectTarget(def, hit);
      targetAdjust.exit();
    },
    cancel(_e, mode) {
      if (mode === 'adjust') finishAdjust();
      pointer = null;
      touching = false;
    }
  });

  function onPointerLeave(): void {
    if (!adjusting && !touching) pointer = null;
  }
</script>

{#if active && def}
  <button
    type="button"
    data-testid="target-surface"
    data-target-ready={reference.grid ? '' : undefined}
    class="absolute z-30 bg-transparent {adjusting ? 'cursor-ns-resize' : 'cursor-crosshair'}"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none;"
    aria-label="{def.label} on the photo"
    {...press.on()}
    onpointerleave={onPointerLeave}
  >
    {#if touching && pointer && img}
      <TouchMagnifier source={img} u={pointer.u} v={pointer.v} width={rect.w} />
    {/if}
    {#if pointer}
      <TargetCallout
        {def}
        hit={adjusting?.hit ?? hover}
        param={adjusting?.session.param ?? null}
        dragging={!!adjusting}
        ready={!!reference.grid}
        failed={reference.failed}
        {touching}
        u={pointer.u}
        v={pointer.v}
      />
    {/if}
  </button>
{/if}
