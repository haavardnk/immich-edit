<script lang="ts">
  import RangeSlider from '#lib/components/editor/controls/RangeSlider.svelte';

  let {
    label,
    name = label,
    min,
    max,
    value,
    unit = '',
    onvalue
  }: {
    label: string;
    name?: string;
    min: number;
    max: number;
    value: number;
    unit?: string;
    onvalue: (value: number) => void;
  } = $props();
</script>

<div class="panel-row h-7 items-center touch:h-11">
  <span class="editor-compact-label select-none">{label}</span>
  <RangeSlider
    label={name}
    {min}
    {max}
    step={1}
    {value}
    oninput={(event) => onvalue((event.currentTarget as HTMLInputElement).valueAsNumber)}
  />
  <span class="px-1 text-right font-mono text-micro tabular-nums text-dark/65 touch:text-xs"
    >{value}{unit}</span
  >
</div>
