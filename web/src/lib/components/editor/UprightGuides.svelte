<script lang="ts">
  import { sourceToStage, stageToSource, type StageFrame } from '$lib/editor/geometryStage';
  import { editor } from '$lib/stores/editor.svelte';
  import type { Point } from '$lib/utils/perspective';
  import { classifyGuide, type UprightGuide } from '$lib/utils/upright';
  import { Icon } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  interface Props {
    frame: StageFrame;
    guides: UprightGuide[];
  }

  let { frame, guides }: Props = $props();

  type End = 'from' | 'to';

  interface Moving {
    index: number;
    end: End;
    at: Point;
    grab: Point;
    dragged: boolean;
  }

  const REMOVE_OFFSET = 16;

  let root = $state<HTMLDivElement | null>(null);
  let drawing = $state<{ from: Point; to: Point } | null>(null);
  let moving = $state<Moving | null>(null);

  const diagonal = $derived(Math.hypot(frame.bboxW, frame.bboxH));
  const lines = $derived(
    guides.map((g, i) => {
      const from = endPoint(i, 'from', g.from);
      const to = endPoint(i, 'to', g.to);
      return { from, to, remove: removeAnchor(from, to) };
    })
  );
  const drawingValid = $derived(
    drawing !== null && classifyGuide(drawing.from, drawing.to, diagonal) !== null
  );

  function endPoint(index: number, end: End, uv: Point): Point {
    if (moving?.index === index && moving.end === end) return moving.at;
    return sourceToStage(frame, uv);
  }

  function removeAnchor(from: Point, to: Point): Point {
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const length = Math.max(Math.hypot(dx, dy), 1e-6);
    return [
      (from[0] + to[0]) / 2 - (dy / length) * REMOVE_OFFSET,
      (from[1] + to[1]) / 2 + (dx / length) * REMOVE_OFFSET
    ];
  }

  function local(e: PointerEvent): Point {
    const rect = root?.getBoundingClientRect();
    return [e.clientX - (rect?.left ?? 0), e.clientY - (rect?.top ?? 0)];
  }

  function capture(e: PointerEvent): boolean {
    e.stopPropagation();
    if (!e.isPrimary || e.button !== 0) return false;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    return true;
  }

  function release(e: PointerEvent): void {
    e.stopPropagation();
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
  }

  function cancel(e: PointerEvent): void {
    e.stopPropagation();
    drawing = null;
    moving = null;
  }

  function startDraw(e: PointerEvent): void {
    if (!capture(e)) return;
    const at = local(e);
    drawing = { from: at, to: at };
  }

  function onDraw(e: PointerEvent): void {
    e.stopPropagation();
    if (!drawing) return;
    drawing = { from: drawing.from, to: local(e) };
  }

  function endDraw(e: PointerEvent): void {
    const drawn = drawing;
    drawing = null;
    if (!drawn) {
      e.stopPropagation();
      return;
    }
    release(e);
    const kind = classifyGuide(drawn.from, drawn.to, diagonal);
    if (!kind) return;
    void editor.addUprightGuide({
      kind,
      from: stageToSource(frame, drawn.from),
      to: stageToSource(frame, drawn.to)
    });
  }

  function startMove(e: PointerEvent, index: number, end: End): void {
    const line = lines[index];
    if (!capture(e) || !line) return;
    const pointer = local(e);
    const at = line[end];
    moving = { index, end, at, grab: [pointer[0] - at[0], pointer[1] - at[1]], dragged: false };
  }

  function onMove(e: PointerEvent): void {
    e.stopPropagation();
    if (!moving) return;
    const pointer = local(e);
    moving = {
      ...moving,
      at: [pointer[0] - moving.grab[0], pointer[1] - moving.grab[1]],
      dragged: true
    };
  }

  function endMove(e: PointerEvent): void {
    release(e);
    const moved = moving;
    const line = moved ? lines[moved.index] : undefined;
    const guide = moved ? guides[moved.index] : undefined;
    moving = null;
    if (!moved?.dragged || !line || !guide) return;
    if (classifyGuide(line.from, line.to, diagonal) !== guide.kind) return;
    const at = stageToSource(frame, moved.at);
    void editor.moveUprightGuide(
      moved.index,
      moved.end === 'from' ? { ...guide, from: at } : { ...guide, to: at }
    );
  }
</script>

<div
  bind:this={root}
  class="absolute inset-0 cursor-crosshair touch-none"
  data-testid="upright-surface"
  role="presentation"
  onpointerdown={startDraw}
  onpointermove={onDraw}
  onpointerup={endDraw}
  onpointercancel={cancel}
>
  <svg
    class="pointer-events-none absolute inset-0"
    width={frame.bboxW}
    height={frame.bboxH}
    aria-hidden="true"
  >
    {#each lines as line, i (i)}
      <line
        x1={line.from[0]}
        y1={line.from[1]}
        x2={line.to[0]}
        y2={line.to[1]}
        stroke="rgb(0 0 0 / 0.6)"
        stroke-width="4"
      />
      <line
        x1={line.from[0]}
        y1={line.from[1]}
        x2={line.to[0]}
        y2={line.to[1]}
        stroke="var(--color-primary)"
        stroke-width="2"
      />
    {/each}
    {#if drawing}
      <line
        x1={drawing.from[0]}
        y1={drawing.from[1]}
        x2={drawing.to[0]}
        y2={drawing.to[1]}
        stroke={drawingValid ? 'var(--color-primary)' : 'rgb(255 255 255 / 0.6)'}
        stroke-width="2"
        stroke-dasharray={drawingValid ? undefined : '4 3'}
      />
    {/if}
  </svg>
  {#each lines as line, i (i)}
    {#each [['from', 'start'], ['to', 'end']] as const as [end, label] (end)}
      {@const [x, y] = line[end]}
      <button
        class="absolute cursor-move rounded-full border border-black/60 bg-primary"
        style="width: 12px; height: 12px; left: {x - 6}px; top: {y - 6}px;"
        onpointerdown={(e) => startMove(e, i, end)}
        onpointermove={onMove}
        onpointerup={endMove}
        onpointercancel={cancel}
        aria-label="upright guide {i + 1} {label}"
      ></button>
    {/each}
    <button
      class="absolute flex cursor-pointer items-center justify-center rounded-full border border-white/40 bg-black/70 text-white"
      style="width: 18px; height: 18px; left: {line.remove[0] - 9}px; top: {line.remove[1] - 9}px;"
      onpointerdown={(e) => e.stopPropagation()}
      onpointerup={(e) => e.stopPropagation()}
      onclick={() => void editor.removeUprightGuide(i)}
      aria-label="Remove guide {i + 1}"
    >
      <Icon icon={mdiClose} size="12px" aria-hidden="true" />
    </button>
  {/each}
</div>
