<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { PRESET_AMOUNT } from '#lib/editor/params/brush.ts';
  import type { AppliedPreset } from '#lib/stores/editor/presetApplied.svelte.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';

  let { applied }: { applied: AppliedPreset } = $props();

  const carries = $derived(applied.carries.geometry || applied.carries.masks);
</script>

<section
  aria-label="Applied preset"
  class="mx-3 flex flex-col gap-1 rounded-xl bg-ghost px-3 pt-1 pb-2"
>
  <p class="flex h-10 items-center gap-2 text-body font-medium text-light-950">
    <span class="truncate">{applied.name}</span>
    <span class="text-label font-normal text-dark/65">applied</span>
  </p>
  <SliderRow
    label="Amount"
    value={applied.amount}
    range={PRESET_AMOUNT}
    onLive={presetStrip.setAmount}
    onCommit={presetStrip.commitAmount}
  />
  {#if carries}
    <div class="flex items-center gap-2">
      <span class="w-20 flex-none text-body text-dark/80 select-none">Include</span>
      {#if applied.carries.geometry}
        <CheckboxRow
          label="Crop"
          look="chip"
          checked={applied.includeGeometry}
          onChange={(v) => presetStrip.setInclude('includeGeometry', v)}
        />
      {/if}
      {#if applied.carries.masks}
        <CheckboxRow
          label="Masks"
          look="chip"
          checked={applied.includeMasks}
          onChange={(v) => presetStrip.setInclude('includeMasks', v)}
        />
      {/if}
    </div>
  {/if}
</section>
