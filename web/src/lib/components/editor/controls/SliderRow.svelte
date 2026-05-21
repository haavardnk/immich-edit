<script lang="ts">
  let {
    label,
    value = $bindable(),
    min,
    max,
    step = 0.1,
    onLive,
    onCommit,
    format = (v: number): string => v.toFixed(2)
  }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onLive: () => void;
    onCommit: () => void;
    format?: (v: number) => string;
  } = $props();

  const isDefault = $derived(value === 0);

  function reset(): void {
    value = 0;
    onCommit();
  }
</script>

<div class="flex flex-col gap-0.5 group">
  <div class="flex items-center justify-between text-[11px] leading-none">
    <button
      class="opacity-70 hover:opacity-100 select-none text-left"
      ondblclick={reset}
      title="double click to reset"
    >
      {label}
    </button>
    <span class="font-mono tabular-nums opacity-60" class:opacity-30={isDefault}>{format(value)}</span>
  </div>
  <input
    type="range"
    class="range range-xs"
    {min}
    {max}
    {step}
    bind:value
    oninput={onLive}
    onchange={onCommit}
  />
</div>
