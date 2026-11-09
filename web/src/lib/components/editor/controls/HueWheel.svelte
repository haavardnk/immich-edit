<script lang="ts">
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { centredPoint, type Point } from '#lib/gestures/point.ts';
  import { tapRecognizer } from '#lib/gestures/tap.ts';

  let {
    hue = $bindable(),
    sat = $bindable(),
    size = 96,
    onLive,
    onCommit,
    commitAction
  }: {
    hue: number;
    sat: number;
    size?: number;
    onLive: () => void;
    onCommit: (action?: string) => void;
    commitAction?: string;
  } = $props();

  let wheel = $state<HTMLDivElement | undefined>();
  let keyboardDirty = $state(false);
  let puck: Point | null = null;
  let lastPointer = 'mouse';

  const cx = $derived(size / 2);
  const cy = $derived(size / 2);
  const r = $derived(size / 2 - 4);

  const thumb = $derived.by(() => {
    const angle = (hue * Math.PI) / 180;
    const radius = (Math.min(Math.max(sat, 0), 100) / 100) * r;
    return { x: cx + radius * Math.cos(angle), y: cy - radius * Math.sin(angle) };
  });

  const hasSat = $derived(sat > 0);
  const valueText = $derived(`hue ${Math.round(hue)}°, saturation ${Math.round(sat)}%`);

  function radius(): number {
    return Math.max((wheel?.clientWidth ?? size) / 2 - 4, 1);
  }

  function place(px: number, py: number): void {
    const dist = Math.sqrt(px * px + py * py);
    const newSat = Math.min(100, Math.round((dist / radius()) * 100));
    let angle = (Math.atan2(-py, px) * 180) / Math.PI;
    if (angle < 0) angle += 360;
    hue = Math.round(angle);
    sat = newSat;
    onLive();
  }

  function updateFromEvent(ev: PointerEvent): void {
    if (!wheel) return;
    const { x, y } = centredPoint(ev, wheel.getBoundingClientRect());
    place(x, y);
  }

  function puckFrom(): Point {
    const angle = (hue * Math.PI) / 180;
    const distance = (Math.min(Math.max(sat, 0), 100) / 100) * radius();
    return { x: distance * Math.cos(angle), y: -distance * Math.sin(angle) };
  }

  function movePuck(stepX: number, stepY: number): void {
    if (!puck) return;
    const next = { x: puck.x + stepX, y: puck.y + stepY };
    const distance = Math.hypot(next.x, next.y);
    const limit = radius();
    puck =
      distance > limit ? { x: (next.x / distance) * limit, y: (next.y / distance) * limit } : next;
    place(puck.x, puck.y);
  }

  function endPuck(): void {
    if (!puck) return;
    puck = null;
    onCommit(commitAction);
  }

  const drag = pointerDrag<true>({
    start(ev) {
      lastPointer = ev.pointerType;
      if (!wheel || ev.pointerType !== 'mouse') return null;
      updateFromEvent(ev);
      return true;
    },
    move: updateFromEvent,
    end: () => onCommit(commitAction)
  });

  const touchDrag = gestures({
    pointerTypes: ['touch', 'pen'],
    recognizers: [
      panRecognizer({
        thresholds: { panSlop: 4 },
        start: () => {
          puck = puckFrom();
        },
        move: (pan) => movePuck(pan.stepX, pan.stepY),
        end: endPuck,
        cancel: endPuck
      }),
      tapRecognizer({ doubleTap: reset })
    ]
  });

  function onDblClick(): void {
    if (lastPointer === 'mouse') reset();
  }

  function reset(): void {
    hue = 0;
    sat = 0;
    onCommit(commitAction);
  }

  function key(ev: KeyboardEvent): void {
    let changed = false;
    if (ev.key === 'ArrowLeft') {
      hue = (hue - 1 + 360) % 360;
      changed = true;
    } else if (ev.key === 'ArrowRight') {
      hue = (hue + 1) % 360;
      changed = true;
    } else if (ev.key === 'ArrowUp') {
      sat = Math.min(100, sat + 1);
      changed = true;
    } else if (ev.key === 'ArrowDown') {
      sat = Math.max(0, sat - 1);
      changed = true;
    }
    if (changed) {
      ev.preventDefault();
      onLive();
      keyboardDirty = true;
    }
  }

  function commitKeyboard(): void {
    if (!keyboardDirty) return;
    keyboardDirty = false;
    onCommit(commitAction);
  }

  function keyUp(ev: KeyboardEvent): void {
    if (!ev.key.startsWith('Arrow')) return;
    commitKeyboard();
  }
</script>

<div
  bind:this={wheel}
  class="hue-wheel"
  style:width="{size}px"
  style:height="{size}px"
  role="slider"
  tabindex="0"
  aria-label="Hue and saturation wheel"
  aria-valuenow={hue}
  aria-valuemin={0}
  aria-valuemax={360}
  aria-valuetext={valueText}
  {...drag.on()}
  {@attach touchDrag}
  ondblclick={onDblClick}
  onkeydown={key}
  onkeyup={keyUp}
  onblur={commitKeyboard}
>
  <div class="wheel-conic"></div>
  <div class="wheel-radial"></div>
  <div class="wheel-ring"></div>
  <div class="wheel-center-dot"></div>
  {#if hasSat}
    <svg class="wheel-line" viewBox="0 0 {size} {size}" aria-hidden="true">
      <line
        x1={cx}
        y1={cy}
        x2={thumb.x}
        y2={thumb.y}
        stroke="var(--color-control-line)"
        stroke-width="1"
      />
    </svg>
  {/if}
  <div class="wheel-thumb" style:left="{thumb.x}px" style:top="{thumb.y}px"></div>
</div>

<style>
  .hue-wheel {
    position: relative;
    border-radius: 50%;
    cursor: crosshair;
    user-select: none;
    touch-action: none;
    outline: none;
    filter: drop-shadow(0 1px 3px var(--color-control-outline));
  }
  .hue-wheel:focus-visible {
    box-shadow: 0 0 0 2px var(--color-primary);
  }
  .wheel-conic {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: var(--gradient-hue-spectrum);
  }
  .wheel-radial {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: radial-gradient(circle, var(--color-control-mid) 0%, transparent 70%);
  }
  .wheel-ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    pointer-events: none;
    box-shadow:
      inset 0 0 0 1px var(--color-control-outline),
      inset 0 0 0 2px color-mix(in srgb, var(--immich-ui-dark) 8%, transparent);
  }
  .wheel-center-dot {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--color-control-line);
    transform: translate(-50%, -50%);
    pointer-events: none;
  }
  .wheel-line {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .wheel-thumb {
    position: absolute;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-image-light);
    border: 2px solid var(--color-slider-thumb-shadow);
    transform: translate(-50%, -50%);
    pointer-events: none;
    box-shadow: 0 1px 3px var(--color-control-shadow);
  }
</style>
