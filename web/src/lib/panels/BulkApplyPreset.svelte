<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { createApplyPresetJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import PresetIncludeToggles from './preset/IncludeToggles.svelte';
  import PresetPicker from './preset/PresetPicker.svelte';
  import { Button } from '@immich/ui';
  import { mdiAutoFix } from '@mdi/js';

  let presetId = $state<string | null>(null);
  let includeGeometry = $state(false);
  let includeMasks = $state(false);
  let busy = $state(false);

  $effect(() => {
    if (!presets.loaded && !presets.loading) void presets.load();
  });

  $effect(() => {
    if (presetId && !presets.presets.some((p) => p.id === presetId)) presetId = null;
  });

  let selected = $derived(presets.presets.find((p) => p.id === presetId) ?? null);

  async function submit(): Promise<void> {
    if (busy || !presetId) return;
    const id = presetId;
    busy = true;
    await runBulkJob(
      (target) => createApplyPresetJob(target, id, { includeGeometry, includeMasks }),
      {
        success: (count) => `Queued preset on ${count} asset${count === 1 ? '' : 's'}`,
        error: 'Failed to queue preset'
      }
    );
    busy = false;
  }
</script>

<div class="flex flex-col gap-2 px-3 py-2">
  <div class="text-[11px] text-dark/65 select-none">
    {selection.targetCount} asset{selection.targetCount === 1 ? '' : 's'} selected
  </div>

  {#if presets.presets.length === 0}
    <div class="text-xs text-dark/65 py-1">No presets yet.</div>
  {:else}
    <PresetPicker bind:selectedId={presetId} />

    <PresetIncludeToggles bind:includeGeometry bind:includeMasks bordered />

    <Button
      size="tiny"
      color="primary"
      fullWidth
      loading={busy}
      leadingIcon={mdiAutoFix}
      disabled={busy || !presetId || selection.targetCount === 0}
      onclick={() => void submit()}
    >
      {selected ? `Apply ${selected.name} to ${selection.targetCount}` : `Select a preset`}
    </Button>

    <p class="text-[10px] leading-snug text-dark/65">
      Runs as a background job. Track progress in the Jobs panel.
    </p>
  {/if}
</div>
