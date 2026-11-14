<script lang="ts">
  import {
    dragBounds,
    dragPosition,
    positionOf,
    valueAt
  } from '#lib/components/editor/controls/relativeDrag.ts';
  import { CHIP, CHIP_ROW } from '#lib/components/editor/touch/chips.ts';
  import Ruler from '#lib/components/editor/touch/Ruler.svelte';
  import { PERSPECTIVE, STRAIGHTEN } from '#lib/editor/params/geometry.ts';
  import type { ParamRange } from '#lib/editor/params/types.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { GeometrySession } from '#lib/stores/editor/geometry.svelte.ts';
  import {
    dialValue,
    holdDial,
    releaseDial,
    setDialValue,
    type DialId
  } from '#lib/stores/editor/geometryDial.ts';
  import { Tabs } from 'bits-ui';

  let { sess, fill = false }: { sess: GeometrySession; fill?: boolean } = $props();

  type Dial = { label: string; range: ParamRange; history: string };

  const DIALS: Record<DialId, Dial> = {
    angle: { label: 'Straighten', range: STRAIGHTEN, history: 'Straighten' },
    vertical: { label: 'Vertical', range: PERSPECTIVE, history: 'Perspective' },
    horizontal: { label: 'Horizontal', range: PERSPECTIVE, history: 'Perspective' },
    aspect: { label: 'Aspect', range: PERSPECTIVE, history: 'Perspective' }
  };
  const ORDER: readonly DialId[] = ['angle', 'vertical', 'horizontal', 'aspect'];

  let active = $state<DialId>('angle');
  let start = 0;
  let position = 0;

  const dial = $derived(DIALS[active]);
  const value = $derived(dialValue(sess, active));
  const bounds = $derived(dragBounds(dial.range));
  const span = $derived(bounds.max - bounds.min || 1);
  const at = $derived((positionOf(value, dial.range) - bounds.min) / span);
  const origin = $derived((positionOf(0, dial.range) - bounds.min) / span);

  function begin(): void {
    start = value;
    position = positionOf(value, dial.range);
  }

  function move(stepX: number, width: number): void {
    position = dragPosition(position, stepX, 0, { ...bounds, width });
    setDialValue(editor, active, valueAt(position, dial.range));
  }

  function end(): void {
    if (dialValue(sess, active) !== start) void editor.commitGeometry(dial.history);
  }

  function hold(id: DialId): void {
    if (id === active) holdDial(editor, id);
  }
</script>

<section
  aria-label="Straighten and perspective"
  class={['flex shrink-0 flex-col', fill && 'flex-1']}
>
  <Tabs.Root
    value={active}
    onValueChange={(next) => (active = next as DialId)}
    activationMode="manual"
    class="py-1.5"
  >
    <Tabs.List aria-label="Dial" class={CHIP_ROW}>
      {#each ORDER as id (id)}
        <Tabs.Trigger
          value={id}
          class={CHIP}
          onpointerdown={() => hold(id)}
          onpointerup={() => releaseDial(editor)}
          onpointercancel={() => releaseDial(editor)}
          onpointerleave={() => releaseDial(editor)}
        >
          {DIALS[id].label}
          {#if Math.abs(dialValue(sess, id)) > 1e-4}
            <span
              data-modified-indicator
              aria-hidden="true"
              class="size-1.5 rounded-full bg-primary"
            ></span>
          {/if}
        </Tabs.Trigger>
      {/each}
    </Tabs.List>
  </Tabs.Root>
  <div class={['flex flex-col', fill && 'my-auto']}>
    <div class="flex items-baseline justify-center gap-2 text-[15px] font-medium text-light-950">
      {dial.label}
      <span class="font-mono text-sm text-primary tabular-nums" data-testid="crop-dial-value"
        >{dial.range.format(value)}</span
      >
    </div>
    <Ruler {at} {origin} onstart={begin} onmove={move} onend={end} testid="crop-dial" />
  </div>
</section>
