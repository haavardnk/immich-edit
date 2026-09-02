<script lang="ts">
  let {
    value,
    min,
    max,
    step = 1,
    label,
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
    label: string;
    disabled?: boolean;
    class?: string;
    gradient?: string;
    onpointerdown?: (event: PointerEvent) => void;
    oninput?: (event: Event) => void;
    onchange?: (event: Event) => void;
    ondblclick?: (event: MouseEvent) => void;
  } = $props();

  const progress = $derived(((value - min) / (max - min)) * 100);
  const background = $derived(
    gradient ??
      `linear-gradient(to right, var(--color-slider-fill) 0%, var(--color-slider-fill) ${progress}%, var(--color-slider-track) ${progress}%, var(--color-slider-track) 100%)`
  );
</script>

<input
  type="range"
  class="slider-range {className}"
  style:background-image={background}
  aria-label={label}
  {min}
  {max}
  {step}
  {disabled}
  {value}
  {onpointerdown}
  {oninput}
  {onchange}
  {ondblclick}
/>
