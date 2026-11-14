<script lang="ts">
  import type { ParamDef } from '#lib/editor/params/types.ts';
  import type { TargetHit, TargetToolDef } from '#lib/editor/targetTools.ts';
  import { levelOf } from '#lib/panels/curves/curvePlot.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { MAGNIFIER_LIFT, MAGNIFIER_SIZE } from './TouchMagnifier.svelte';

  let {
    def,
    hit,
    param,
    dragging,
    ready,
    failed,
    touching,
    u,
    v
  }: {
    def: TargetToolDef;
    hit: TargetHit | null;
    param: ParamDef | null;
    dragging: boolean;
    ready: boolean;
    failed: boolean;
    touching: boolean;
    u: number;
    v: number;
  } = $props();

  const title = $derived.by(() => {
    if (!hit) return null;
    if (hit.kind === 'param') return hit.name;
    const point =
      dragging && curveEditor.selected !== null
        ? editor.edits.basic.curves[hit.channel][curveEditor.selected]
        : null;
    return point ? `In ${levelOf(point.x)} · Out ${levelOf(point.y)}` : `In ${levelOf(hit.level)}`;
  });
  const action = $derived.by(() => {
    if (!hit || touching) return null;
    if (dragging)
      return param ? `${param.label} ${param.format(param.get(editor.edits, editor.isRaw))}` : null;
    return hit.kind === 'param' ? `drag ↕ ${hit.axes.main.label}` : 'drag ↕ Out';
  });
  const keys = $derived.by(() => {
    if (!hit || touching || dragging) return null;
    const parts = [];
    if (hit.kind === 'param' && hit.axes.hue) parts.push(`${keyLabel('Alt')} drag ↕ Hue`);
    if (hit.kind === 'param' && hit.axes.lightness) {
      parts.push(`${keyLabel('Shift')} ${hit.axes.lightness.label}`);
    }
    parts.push(`${keysFor('editorEscape')} exit`);
    return parts.join(' · ');
  });
  const empty = $derived(
    failed ? 'Could not read the photo' : !ready ? 'Sampling…' : `No ${def.noun} here`
  );
  const shift = $derived(
    touching
      ? `-50%, calc(-100% - ${MAGNIFIER_LIFT + MAGNIFIER_SIZE + 8}px)`
      : `${u > 0.7 ? 'calc(-100% - 16px)' : '16px'}, ${v > 0.8 ? 'calc(-100% - 16px)' : '16px'}`
  );
</script>

<span
  data-testid="target-callout"
  class="pointer-events-none absolute flex flex-col gap-0.5 rounded bg-black/75 px-2.5 py-1.5 text-left text-[11px] whitespace-nowrap text-white shadow-lg tabular-nums touch:text-[13px]"
  style="left: {u * 100}%; top: {v * 100}%; transform: translate({shift});"
>
  {#if title}
    <span>
      <b class="font-semibold">{title}</b>{#if action}<span class="text-white/80"
          >{` · ${action}`}</span
        >{/if}
    </span>
    {#if keys}
      <span class="text-white/60">{keys}</span>
    {/if}
  {:else}
    <span class="text-white/60">{empty}</span>
  {/if}
</span>
