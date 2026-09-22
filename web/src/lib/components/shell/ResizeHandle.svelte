<script lang="ts">
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

  let pointerId: number | null = null;
  let startPosition = 0;
  let startValue = 0;
  const dragging = $derived(pointerId !== null);

  function position(event: PointerEvent): number {
    return orientation === 'horizontal' ? event.clientX : event.clientY;
  }

  function start(event: PointerEvent): void {
    if (pointerId !== null) return;
    pointerId = event.pointerId;
    startPosition = position(event);
    startValue = value;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function move(event: PointerEvent): void {
    if (event.pointerId !== pointerId) return;
    const delta = startPosition - position(event);
    onLive(startValue + (grow === 'start' ? delta : -delta));
  }

  function finish(event: PointerEvent): void {
    if (event.pointerId !== pointerId) return;
    pointerId = null;
    const target = event.currentTarget as HTMLElement;
    if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId);
    onCommit();
  }

  function resizeWithKeyboard(event: KeyboardEvent): void {
    const amount = event.shiftKey ? shiftStep : step;
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
  onpointerdown={start}
  onpointermove={move}
  onpointerup={finish}
  onpointercancel={finish}
  onlostpointercapture={finish}
  onkeydown={resizeWithKeyboard}
></div>
