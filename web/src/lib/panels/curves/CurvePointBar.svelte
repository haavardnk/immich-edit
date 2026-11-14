<script lang="ts">
  import SliderValue from '#lib/components/editor/controls/SliderValue.svelte';
  import { amountRange } from '#lib/editor/params/range.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { CurveChannel, CurvePoint } from '#lib/types/edits.ts';
  import { Button } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { fromLevel, levelOf } from './curvePlot';
  import { CHANNEL_LABELS, dragPoint, removePoint } from './curves';

  const LEVELS = amountRange(255);

  let { channel, selected = $bindable(null) }: { channel: CurveChannel; selected?: number | null } =
    $props();

  const points = $derived(editor.edits.basic.curves[channel]);
  const point = $derived(selected === null ? null : (points[selected] ?? null));
  const interior = $derived(selected !== null && selected > 0 && selected < points.length - 1);

  function place(next: CurvePoint): void {
    if (selected === null) return;
    const moved = dragPoint(points, selected, next);
    if (!moved) return;
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: moved };
    editor.onLive();
  }

  function commit(): void {
    void editor.onCommit(`Curve ${CHANNEL_LABELS[channel]}`);
  }

  function remove(): void {
    const next = selected === null ? null : removePoint(points, selected);
    if (!next) return;
    editor.edits.basic.curves = { ...editor.edits.basic.curves, [channel]: next };
    selected = null;
    commit();
  }
</script>

<div class="flex h-7 items-center gap-1 text-[11px] text-dark/65 touch:h-11 touch:text-[13px]">
  {#if point}
    <span>In</span>
    <div class="w-9 touch:w-11">
      <SliderValue
        label="In"
        value={levelOf(point.x)}
        range={LEVELS}
        resetValue={levelOf(point.x)}
        modified={false}
        disabled={!interior}
        onLive={(level) => place({ x: fromLevel(level), y: point.y })}
        onCommit={commit}
      />
    </div>
    <span class="ml-2">Out</span>
    <div class="w-9 touch:w-11">
      <SliderValue
        label="Out"
        value={levelOf(point.y)}
        range={LEVELS}
        resetValue={levelOf(point.x)}
        modified={false}
        disabled={false}
        onLive={(level) => place({ x: point.x, y: fromLevel(level) })}
        onCommit={commit}
      />
    </div>
  {:else}
    <span class="touch:hidden">Click the curve to add a point</span>
    <span class="hidden touch:inline">Tap the curve to add a point</span>
  {/if}
  <Button
    size="small"
    variant="ghost"
    color="secondary"
    leadingIcon={mdiClose}
    class="ml-auto h-6 rounded-full px-2 text-[11px] touch:h-9 touch:bg-ghost touch:px-3.5 touch:text-[13px]"
    disabled={!interior}
    onclick={remove}
  >
    Delete point
  </Button>
</div>
