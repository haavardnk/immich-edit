<script lang="ts">
  import { percent, percentRange } from '#lib/utils/percent.ts';
  import DualRange from './DualRange.svelte';

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
</script>

<div
  class="panel-row h-7 items-center rounded-sm px-0.5 transition-colors hover:bg-light-950/3 {disabled
    ? 'pointer-events-none opacity-50'
    : ''}"
>
  <span class="truncate text-[11px] font-medium text-dark/65 select-none">{label}</span>
  <DualRange
    {value}
    {thumbLabels}
    {defaultValue}
    {disabled}
    {onLive}
    {onCommit}
    valueText={(v) => `${percent(v)}% of the photo is nearer`}
  />
  <span class="px-1 text-right font-mono text-[10px] text-dark/65 tabular-nums">
    {percentRange(value[0], value[1])}
  </span>
</div>
