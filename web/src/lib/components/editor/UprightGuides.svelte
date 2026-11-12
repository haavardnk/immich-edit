<script lang="ts">
  import { sourceToStage, stageToSource, type StageFrame } from '#lib/editor/geometryStage.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { Point } from '#lib/utils/perspective.ts';
  import { classifyGuide, type GuideKind, type UprightGuide } from '#lib/utils/upright.ts';
  import {
    drawingHint,
    guideLabels,
    isGuideLength,
    tiltFromLevel
  } from '#lib/utils/uprightFeedback.ts';
  import GuideCounts from './crop/GuideCounts.svelte';
  import { Icon } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { elementPoint } from '#lib/gestures/point.ts';
  import { HANDLE_DOT } from '#lib/editor/handles.ts';
  import { handleHit } from '#lib/editor/handleHit.svelte.ts';

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
  const LABEL_OFFSET = 14;
  const STROKE: Record<GuideKind, string> = {
    vertical: 'var(--color-guide-vertical)',
    horizontal: 'var(--color-guide-horizontal)'
  };
  const hit = $derived(handleHit());

  let root = $state<HTMLDivElement | null>(null);
  let drawing = $state<{ from: Point; to: Point } | null>(null);
  let moving = $state<Moving | null>(null);

  const diagonal = $derived(Math.hypot(frame.bboxW, frame.bboxH));
  const labels = $derived(guideLabels(guides));
  const lines = $derived(
    guides.map((g, i) => {
      const from = endPoint(i, 'from', g.from);
      const to = endPoint(i, 'to', g.to);
      return {
        from,
        to,
        kind: g.kind,
        label: labels[i] ?? '',
        remove: sideAnchor(from, to, REMOVE_OFFSET),
        tag: sideAnchor(from, to, -LABEL_OFFSET)
      };
    })
  );
  const drawingKind = $derived(drawing ? classifyGuide(drawing.from, drawing.to, diagonal) : null);
  const hint = $derived(
    drawing ? drawingHint(guides, drawing.from, drawing.to, drawingKind, diagonal) : null
  );

  function endPoint(index: number, end: End, uv: Point): Point {
    if (moving?.index === index && moving.end === end) return moving.at;
    return sourceToStage(frame, uv);
  }

  function sideAnchor(from: Point, to: Point, offset: number): Point {
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const length = Math.max(Math.hypot(dx, dy), 1e-6);
    return [
      (from[0] + to[0]) / 2 - (dy / length) * offset,
      (from[1] + to[1]) / 2 + (dx / length) * offset
    ];
  }

  function local(e: PointerEvent): Point {
    const at = root ? elementPoint(e, root) : { x: e.clientX, y: e.clientY };
    return [at.x, at.y];
  }

  function primary(e: PointerEvent): boolean {
    e.stopPropagation();
    if (!e.isPrimary || e.button !== 0) return false;
    e.preventDefault();
    return true;
  }

  function cancelDrag(e: PointerEvent): void {
    e.stopPropagation();
    drawing = null;
    moving = null;
  }

  const draw = pointerDrag<true>({
    start(e) {
      if (!primary(e)) return null;
      const at = local(e);
      drawing = { from: at, to: at };
      return true;
    },
    move(e) {
      e.stopPropagation();
      if (!drawing) return;
      drawing = { from: drawing.from, to: local(e) };
    },
    end(e) {
      e.stopPropagation();
      const drawn = drawing;
      drawing = null;
      if (!drawn) return;
      const kind = classifyGuide(drawn.from, drawn.to, diagonal);
      if (!kind) {
        if (isGuideLength(drawn.from, drawn.to, diagonal)) {
          editor.noteDroppedUprightGuide(tiltFromLevel(drawn.from, drawn.to));
        }
        return;
      }
      void editor.addUprightGuide({
        kind,
        from: stageToSource(frame, drawn.from),
        to: stageToSource(frame, drawn.to)
      });
    },
    cancel: cancelDrag
  });

  const move = pointerDrag<true, { index: number; end: End }>({
    start(e, { index, end }) {
      const line = lines[index];
      if (!primary(e) || !line) return null;
      const pointer = local(e);
      const at = line[end];
      moving = { index, end, at, grab: [pointer[0] - at[0], pointer[1] - at[1]], dragged: false };
      return true;
    },
    move(e) {
      e.stopPropagation();
      if (!moving) return;
      const pointer = local(e);
      moving = {
        ...moving,
        at: [pointer[0] - moving.grab[0], pointer[1] - moving.grab[1]],
        dragged: true
      };
    },
    end(e) {
      e.stopPropagation();
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
    },
    cancel: cancelDrag
  });
</script>

<div
  bind:this={root}
  class="absolute inset-0 cursor-crosshair touch-none"
  data-testid="upright-surface"
  role="presentation"
  {...draw.on()}
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
        stroke={STROKE[line.kind]}
        stroke-width="2"
      />
    {/each}
    {#if drawing}
      <line
        x1={drawing.from[0]}
        y1={drawing.from[1]}
        x2={drawing.to[0]}
        y2={drawing.to[1]}
        stroke={drawingKind ? STROKE[drawingKind] : 'rgb(255 255 255 / 0.6)'}
        stroke-width="2"
        stroke-dasharray={drawingKind ? undefined : '4 3'}
      />
    {/if}
  </svg>
  <span
    class="pointer-events-none absolute top-3 left-1/2 z-20 -translate-x-1/2 rounded-full bg-black/70 px-3 py-1 text-xs whitespace-nowrap text-white shadow-lg backdrop-blur-sm"
    data-testid="upright-counts"><GuideCounts {guides} /></span
  >
  {#each lines as line, i (i)}
    <span
      class="pointer-events-none absolute -translate-1/2 font-mono text-[11px] font-semibold text-shadow-sm {line.kind ===
      'vertical'
        ? 'text-guide-vertical'
        : 'text-guide-horizontal'}"
      style="left: {line.tag[0]}px; top: {line.tag[1]}px;"
      data-testid="upright-label">{line.label}</span
    >
    {#each [['from', 'start'], ['to', 'end']] as const as [end, label] (end)}
      {@const [x, y] = line[end]}
      <button
        class="absolute cursor-move"
        style="width: {hit * 2}px; height: {hit * 2}px; left: {x - hit}px; top: {y - hit}px;"
        {...move.on({ index: i, end })}
        aria-label="upright guide {i + 1} {label}"
      >
        <span
          class="pointer-events-none absolute rounded-full border border-black/60 bg-white"
          style="inset: {hit - HANDLE_DOT.guideEnd}px;"
        ></span>
      </button>
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
  {#if drawing && hint}
    <span
      role="status"
      class="pointer-events-none absolute z-20 rounded-full bg-black/70 px-2 py-0.5 text-xs whitespace-nowrap text-white"
      style="left: {drawing.to[0] + 14}px; top: {drawing.to[1] + 10}px;"
      data-testid="upright-hint">{hint}</span
    >
  {/if}
</div>
