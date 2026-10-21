<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import RangeSlider from '$lib/components/editor/controls/RangeSlider.svelte';
  import type { ApplyPresetOptions } from '$lib/api/jobs';
  import { LOOK_AMOUNT_FULL, LOOK_AMOUNT_MAX } from '$lib/edits/lookAmount';

  let {
    options = $bindable(),
    bordered = false,
    inline = false
  }: {
    options: ApplyPresetOptions;
    bordered?: boolean;
    inline?: boolean;
  } = $props();
</script>

{#snippet includes()}
  <CheckboxRow
    label="Geometry & crop"
    checked={options.includeGeometry}
    onChange={(v) => (options.includeGeometry = v)}
  />
  <CheckboxRow
    label="Masks"
    checked={options.includeMasks}
    onChange={(v) => (options.includeMasks = v)}
  />
{/snippet}

<div
  class="flex flex-col text-[11px] text-dark/65 {inline ? '' : 'gap-1'} {bordered
    ? 'border-t border-dark/10 pt-3'
    : ''}"
>
  <div class="panel-row h-7 items-center">
    <span class="editor-compact-label select-none">Amount</span>
    <RangeSlider
      label="Preset amount"
      min={0}
      max={LOOK_AMOUNT_MAX}
      step={1}
      defaultValue={LOOK_AMOUNT_FULL}
      value={options.amount}
      valueText="{options.amount}%"
      title="Double click to reset"
      oninput={(event) => {
        options.amount = (event.currentTarget as HTMLInputElement).valueAsNumber;
      }}
      ondblclick={() => (options.amount = LOOK_AMOUNT_FULL)}
    />
    <span class="px-1 text-right font-mono text-[10px] tabular-nums text-dark/65"
      >{options.amount}%</span
    >
  </div>
  {#if inline}
    <div class="flex items-center gap-3">
      <span class="editor-compact-label select-none">Include</span>
      {@render includes()}
    </div>
  {:else}
    <span class="uppercase tracking-wider text-[10px] text-dark/65">Apply includes</span>
    {@render includes()}
  {/if}
</div>
