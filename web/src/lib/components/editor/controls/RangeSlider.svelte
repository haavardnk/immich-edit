<script lang="ts">
  import { arrowStepValue } from './sliderKeys';

  let {
    value,
    min,
    max,
    step = 1,
    coarseStep = step * 10,
    defaultValue,
    label,
    valueText,
    disabled = false,
    class: className = '',
    gradient,
    onpointerdown,
    oninput,
    onchange,
    ondblclick
  }: {
    value: number;
    min: number;
    max: number;
    step?: number;
    coarseStep?: number;
    defaultValue?: number;
    label: string;
    valueText?: string;
    disabled?: boolean;
    class?: string;
    gradient?: string;
    onpointerdown?: (event: PointerEvent) => void;
    oninput?: (event: Event) => void;
    onchange?: (event: Event) => void;
    ondblclick?: (event: MouseEvent) => void;
  } = $props();

  let input = $state<HTMLInputElement | null>(null);

  const progress = $derived(((value - min) / (max - min)) * 100);
  const fill = $derived(
    gradient ??
      `linear-gradient(to right, var(--color-slider-fill) 0%, var(--color-slider-fill) ${progress}%, var(--color-slider-track) ${progress}%, var(--color-slider-track) 100%)`
  );
  const markerAt = $derived(
    defaultValue !== undefined && defaultValue > min && defaultValue < max
      ? ((defaultValue - min) / (max - min)) * 100
      : null
  );
  const background = $derived(
    markerAt === null
      ? fill
      : `linear-gradient(var(--color-slider-marker), var(--color-slider-marker)), ${fill}`
  );

  function onKeyDown(event: KeyboardEvent): void {
    if (disabled || !input) return;
    const next = arrowStepValue(event, { value, min, max, step, coarseStep });
    if (next === null || next === value) return;
    event.preventDefault();
    input.value = String(next);
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
  }
</script>

<input
  bind:this={input}
  type="range"
  class="slider-range {className}"
  style:background-image={background}
  style:background-size={markerAt === null ? undefined : '1px 6px, 100% 2px'}
  style:background-position={markerAt === null
    ? undefined
    : `calc(${markerAt}% - 0.5px) center, center`}
  aria-label={label}
  aria-valuetext={valueText}
  {min}
  {max}
  {step}
  {disabled}
  {value}
  {onpointerdown}
  {oninput}
  {onchange}
  {ondblclick}
  onkeydown={onKeyDown}
/>
