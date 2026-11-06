<script lang="ts">
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { LOOK_AMOUNT_FULL } from '#lib/edits/lookAmount.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import PresetApplyOptions from '#lib/components/preset/ApplyOptions.svelte';
  import PresetPicker from '#lib/components/preset/PresetPicker.svelte';
  import { Button } from '@immich/ui';
  import { mdiAutoFix } from '@mdi/js';

  let presetId = $state<string | null>(null);
  let applyOptions = $state<ApplyPresetOptions>({
    includeGeometry: false,
    includeMasks: false,
    amount: LOOK_AMOUNT_FULL
  });
  $effect(() => {
    if (!presets.loaded && !presets.loading) void presets.load();
  });

  $effect(() => {
    if (presetId && !presets.presets.some((p) => p.id === presetId)) presetId = null;
  });

  let selected = $derived(presets.presets.find((p) => p.id === presetId) ?? null);

  async function submit(): Promise<void> {
    if (!presetId) return;
    await bulkActions.applyPreset(presetId, $state.snapshot(applyOptions));
  }
</script>

<div class="flex flex-col gap-2 px-3 py-2">
  <div class="text-[11px] text-dark/65 select-none">
    {countOf(selection.count, 'asset')} selected
  </div>

  {#if presets.presets.length === 0}
    <div class="text-xs text-dark/65 py-1">No presets yet.</div>
  {:else}
    <PresetPicker bind:selectedId={presetId} />

    {#if selected}
      <PresetApplyOptions bind:options={applyOptions} bordered />

      <Button
        size="tiny"
        color="primary"
        fullWidth
        loading={bulkActions.busy}
        leadingIcon={mdiAutoFix}
        disabled={bulkActions.metaBusy || selection.count === 0}
        onclick={() => void submit()}
      >
        Apply {selected.name} to {selection.count}
      </Button>

      <p class="text-[10px] leading-snug text-dark/65">
        Runs as a background job. Track progress in the Jobs panel.
      </p>
    {/if}
  {/if}
</div>
