<script lang="ts">
  import { Slider } from 'bits-ui';

  let {
    value,
    min,
    max,
    step,
    thumbLabels,
    valueText,
    defaultValue,
    onLive,
    onCommit
  }: {
    value: [number, number];
    min: number;
    max: number;
    step: number;
    thumbLabels: [string, string];
    valueText: (value: number) => string;
    defaultValue: [number, number];
    onLive: (value: [number, number]) => void;
    onCommit: () => void;
  } = $props();

  function pair(next: number[]): [number, number] {
    return [next[0] ?? min, next[1] ?? max];
  }

  function reset(): void {
    if (value[0] === defaultValue[0] && value[1] === defaultValue[1]) return;
    onLive(defaultValue);
    onCommit();
  }
</script>

<Slider.Root
  type="multiple"
  class="dual-range"
  value={[...value]}
  {min}
  {max}
  {step}
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
        aria-valuetext={valueText(value[thumb.index] ?? min)}
      />
    {/each}
  {/snippet}
</Slider.Root>
