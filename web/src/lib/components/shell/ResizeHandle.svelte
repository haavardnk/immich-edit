<script lang="ts">
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  type Orientation = 'horizontal' | 'vertical';
  type Grow = 'start' | 'end';

  let {
    label,
    orientation,
    grow = 'start',
    value,
    min,
    max,
    step,
    shiftStep,
    class: className = '',
    activeClass = '',
    onLive,
    onCommit
  }: {
    label: string;
    orientation: Orientation;
    grow?: Grow;
    value: number;
    min: number;
    max: number;
    step: number;
    shiftStep: number;
    class?: string;
    activeClass?: string;
    onLive: (value: number) => void;
    onCommit: () => void;
  } = $props();

  let dragging = $state(false);

  function position(event: PointerEvent): number {
    return orientation === 'horizontal' ? event.clientX : event.clientY;
  }

  const resize = pointerDrag({
    start: (event) => {
      dragging = true;
      return { position: position(event), value };
    },
    move: (event, from) => {
      const delta = from.position - position(event);
      onLive(from.value + (grow === 'start' ? delta : -delta));
    },
    end: () => {
      dragging = false;
      onCommit();
    }
  });

  function resizeWithKeyboard(event: KeyboardEvent): void {
    const amount = held('coarse', event) ? shiftStep : step;
    const towardStart = orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp';
    const towardEnd = orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown';
    const increaseKey = grow === 'start' ? towardStart : towardEnd;
    const decreaseKey = grow === 'start' ? towardEnd : towardStart;
    if (event.key === increaseKey) onLive(value + amount);
    else if (event.key === decreaseKey) onLive(value - amount);
    else if (event.key === 'Home') onLive(min);
    else if (event.key === 'End') onLive(max);
    else return;
    onCommit();
    event.preventDefault();
  }
</script>

<div
  role="slider"
  tabindex="0"
  aria-label={label}
  aria-orientation={orientation}
  aria-valuemin={min}
  aria-valuemax={max}
  aria-valuenow={value}
  class="touch-none {className} {dragging ? activeClass : ''}"
  {...resize.on()}
  onkeydown={resizeWithKeyboard}
></div>
