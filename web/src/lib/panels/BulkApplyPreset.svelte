<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { createApplyPresetJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import Icon from '$lib/components/Icon.svelte';
  import PresetIncludeToggles from './preset/IncludeToggles.svelte';
  import PresetPicker from './preset/PresetPicker.svelte';
  import { mdiAutoFix, mdiLoading } from '@mdi/js';

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
        error: 'Failed to queue preset',
      },
    );
    busy = false;
  }
</script>

<div class="flex flex-col gap-4 px-4 pt-1 pb-1">
  <div class="text-[11px] text-immich-dark-fg/60 select-none">
    {selection.targetCount} asset{selection.targetCount === 1 ? '' : 's'} selected
  </div>

  {#if presets.presets.length === 0}
    <div class="text-xs text-immich-dark-fg/30 py-1">No presets yet.</div>
  {:else}
    <PresetPicker bind:selectedId={presetId} />

    <PresetIncludeToggles
      bind:includeGeometry
      bind:includeMasks
      bordered
    />

    <button
      class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      disabled={busy || !presetId || selection.targetCount === 0}
      onclick={() => void submit()}
    >
      {#if busy}
        <Icon path={mdiLoading} size={16} class="animate-spin" />
      {:else}
        <Icon path={mdiAutoFix} size={16} />
      {/if}
      {selected ? `Apply ${selected.name} to ${selection.targetCount}` : `Select a preset`}
    </button>

    <p class="text-[11px] leading-relaxed text-immich-dark-fg/40">
      Runs as a background job. Track progress in the Jobs panel.
    </p>
  {/if}

  <div class="h-4"></div>
</div>
