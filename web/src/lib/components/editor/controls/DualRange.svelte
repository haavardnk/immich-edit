<script lang="ts">
  import { Slider } from 'bits-ui';

  const MIN = 0;
  const MAX = 1;
  const STEP = 0.01;

  let {
    value,
    thumbLabels,
    valueText,
    defaultValue,
    disabled = false,
    trackClass = '',
    onLive,
    onCommit
  }: {
    value: [number, number];
    thumbLabels: [string, string];
    valueText: (value: number) => string;
    defaultValue: [number, number];
    disabled?: boolean;
    trackClass?: string;
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
    <span class="dual-range-track {trackClass}">
      <Slider.Range class="dual-range-fill" />
    </span>
    {#each thumbItems as thumb (thumb.index)}
      <Slider.Thumb
        index={thumb.index}
        class="dual-range-thumb"
        aria-label={thumbLabels[thumb.index]}
        aria-valuetext={valueText(value[thumb.index] ?? MIN)}
      />
    {/each}
  {/snippet}
</Slider.Root>
