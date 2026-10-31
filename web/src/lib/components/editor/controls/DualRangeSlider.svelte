<script lang="ts">
  import { percent, percentRange } from '#lib/utils/percent.ts';
  import { Slider } from 'bits-ui';

  const MIN = 0;
  const MAX = 1;
  const STEP = 0.01;

  let {
    label,
    value,
    thumbLabels,
    defaultValue,
    disabled = false,
    onLive,
    onCommit
  }: {
    label: string;
    value: [number, number];
    thumbLabels: [string, string];
    defaultValue: [number, number];
    disabled?: boolean;
    onLive: (value: [number, number]) => void;
    onCommit: () => void;
  } = $props();

  function pair(next: number[]): [number, number] {
    return [next[0] ?? MIN, next[1] ?? MAX];
  }

  function reset(): void {
    if (disabled) return;
    if (value[0] === defaultValue[0] && value[1] === defaultValue[1]) return;
    onLive(defaultValue);
    onCommit();
  }
</script>

<div
  class="panel-row h-7 items-center rounded-sm px-0.5 transition-colors hover:bg-white/3 {disabled
    ? 'pointer-events-none opacity-50'
    : ''}"
>
  <span class="truncate text-[11px] font-medium text-dark/65 select-none">{label}</span>
  <Slider.Root
    type="multiple"
    class="dual-range"
    value={[...value]}
    min={MIN}
    max={MAX}
    step={STEP}
    {disabled}
    onValueChange={(next: number[]) => onLive(pair(next))}
    onValueCommit={onCommit}
    ondblclick={reset}
  >
    {#snippet children({ thumbItems })}
      <span class="dual-range-track">
        <Slider.Range class="dual-range-fill" />
      </span>
      {#each thumbItems as thumb (thumb.index)}
        <Slider.Thumb
          index={thumb.index}
          class="dual-range-thumb"
          aria-label={thumbLabels[thumb.index]}
          aria-valuetext="{percent(value[thumb.index] ?? MIN)}% of the photo is nearer"
        />
      {/each}
    {/snippet}
  </Slider.Root>
  <span class="px-1 text-right font-mono text-[10px] text-dark/65 tabular-nums">
    {percentRange(value[0], value[1])}
  </span>
</div>
