<script lang="ts">
  import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import RangeSlider from '#lib/components/editor/controls/RangeSlider.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { LOOK_AMOUNT_FULL, LOOK_AMOUNT_MAX } from '#lib/edits/lookAmount.ts';

  let {
    options = $bindable(),
    bordered = false,
    inline = false,
    amount,
    note = null,
    onamount,
    onamountcommit
  }: {
    options: ApplyPresetOptions;
    bordered?: boolean;
    inline?: boolean;
    amount?: number;
    note?: string | null;
    onamount?: (value: number) => void;
    onamountcommit?: () => void;
  } = $props();

  const value = $derived(amount ?? options.amount);

  function setAmount(next: number): void {
    if (onamount) onamount(next);
    else options.amount = next;
  }

  const short = $derived(inline && layout.mode === 'touch');
  const includeClass =
    'touch:h-9 touch:gap-2 touch:rounded-full touch:bg-ghost touch:px-3 touch:text-[13px] touch:text-dark';
</script>

{#snippet includes()}
  <CheckboxRow
    label={short ? 'Crop' : 'Geometry & crop'}
    class={inline ? includeClass : ''}
    checked={options.includeGeometry}
    onChange={(v) => (options.includeGeometry = v)}
  />
  <CheckboxRow
    label="Masks"
    class={inline ? includeClass : ''}
    checked={options.includeMasks}
    onChange={(v) => (options.includeMasks = v)}
  />
{/snippet}

<div
  class="flex flex-col text-[11px] text-dark/65 {inline ? 'touch:gap-1' : 'gap-1'} {bordered
    ? 'border-t border-dark/10 pt-3'
    : ''}"
>
  <div class="panel-row h-7 items-center touch:h-11 touch:grid-cols-[6rem_minmax(0,1fr)_3.25rem]">
    <span class="editor-compact-label flex items-center gap-1.5 select-none touch:text-[13px]">
      {#if value !== LOOK_AMOUNT_FULL}
        <span class="hidden size-1.5 rounded-full bg-primary touch:block" aria-hidden="true"></span>
      {/if}
      Amount
    </span>
    <RangeSlider
      label="Preset amount"
      min={0}
      max={LOOK_AMOUNT_MAX}
      step={1}
      defaultValue={LOOK_AMOUNT_FULL}
      {value}
      valueText="{value}%"
      title="Double click to reset"
      oninput={(event) => setAmount((event.currentTarget as HTMLInputElement).valueAsNumber)}
      onchange={onamountcommit}
      ondblclick={() => {
        setAmount(LOOK_AMOUNT_FULL);
        onamountcommit?.();
      }}
    />
    <span
      class="px-1 text-right font-mono text-[10px] tabular-nums text-dark/65 touch:rounded touch:bg-light-200 touch:py-1 touch:text-xs touch:text-dark"
      >{value}%</span
    >
  </div>
  {#if note}
    <p class="truncate text-[11px] text-dark/65 touch:text-[13px]">{note}</p>
  {/if}
  {#if inline}
    <div class="flex items-center gap-3 touch:gap-2">
      <span class="editor-compact-label select-none touch:w-24 touch:flex-none touch:text-[13px]"
        >Include</span
      >
      {@render includes()}
    </div>
  {:else}
    <span class="uppercase tracking-wider text-[10px] text-dark/65">Apply includes</span>
    {@render includes()}
  {/if}
</div>
