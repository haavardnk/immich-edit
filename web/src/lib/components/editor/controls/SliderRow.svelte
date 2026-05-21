<script lang="ts">
  let {
    label,
    value = $bindable(),
    min,
    max,
    step = 0.1,
    onLive,
    onCommit,
    format = (v: number): string => v.toFixed(2),
    gradient
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onLive: () => void;
    onCommit: () => void;
    format?: (v: number) => string;
    gradient?: string;
  } = $props();

  const isDefault = $derived(value === 0);

  function reset(): void {
    value = 0;
    onCommit();
  }
</script>

<div class="flex flex-col gap-1 group">
  <div class="flex items-center justify-between text-[11px] leading-none">
    <button
      class="text-immich-dark-fg/60 hover:text-immich-dark-fg transition-colors select-none text-left"
      ondblclick={reset}
      title="double click to reset"
    >
      {label}
    </button>
    <span
      class="font-mono tabular-nums text-[10px] transition-opacity {isDefault ? 'text-immich-dark-fg/20' : 'text-immich-dark-fg/50'}"
    >
      {format(value)}
    </span>
  </div>
  <input
    type="range"
    class="slider-range"
    style:background={gradient}
    {min}
    {max}
    {step}
    bind:value
    oninput={onLive}
    onchange={onCommit}
    ondblclick={reset}
  />
</div>

<style>
  .slider-range {
    width: 100%;
    height: 4px;
    border-radius: 9999px;
    appearance: none;
    cursor: pointer;
    background: rgba(255, 255, 255, 0.1);
  }
  .slider-range::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: rgb(var(--immich-dark-primary));
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    transition: transform 0.15s;
  }
  .slider-range::-webkit-slider-thumb:hover {
    transform: scale(1.25);
  }
  .slider-range::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: rgb(var(--immich-dark-primary));
    border: 0;
  }
</style>
