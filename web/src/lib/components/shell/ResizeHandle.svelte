<script lang="ts">
  type Orientation = 'horizontal' | 'vertical';

  let {
    label,
    orientation,
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
    onLive(startValue + startPosition - position(event));
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
    const increaseKey = orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp';
    const decreaseKey = orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown';
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
