<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { jobs } from '$lib/stores/jobs.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { createApplyPresetJob } from '$lib/api/jobs';
  import { toasts } from '$lib/stores/toasts.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import PresetIncludeToggles from './preset/IncludeToggles.svelte';
  import { mdiAutoFix, mdiLoading } from '@mdi/js';

  let presetId = $state<string | null>(null);
  let includeGeometry = $state(false);
  let includeMasks = $state(false);
  let includeOutput = $state(false);
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
    const ids = [...selection.selected];
    if (ids.length === 0) return;
    busy = true;
    try {
      await createApplyPresetJob(ids, presetId, { includeGeometry, includeMasks, includeOutput });
      toasts.push(
        'success',
        `Queued preset on ${ids.length} asset${ids.length === 1 ? '' : 's'}`,
        4000,
      );
      if (jobs.open) {
        void jobs.load();
      } else {
        jobs.toggle();
      }
    } catch (e) {
      toasts.push('error', `Failed to queue preset: ${(e as Error).message}`, 6000);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex flex-col gap-4 px-4 pt-3">
  <div class="text-[11px] text-immich-dark-fg/60 select-none">
    {selection.count} asset{selection.count === 1 ? '' : 's'} selected
  </div>

  {#if presets.presets.length === 0}
    <div class="text-xs text-immich-dark-fg/30 py-1">No presets yet.</div>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each presets.grouped as { group, items } (group)}
        {#if group}
          <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40 pt-1">{group}</div>
        {/if}
        {#each items as p (p.id)}
          <button
            class="text-left px-2.5 py-1.5 rounded-lg text-xs transition-colors truncate {presetId ===
            p.id
              ? 'bg-immich-dark-primary/25 text-immich-dark-primary'
              : 'bg-white/5 hover:bg-white/10'}"
            onclick={() => (presetId = p.id)}
          >
            {p.name}
          </button>
        {/each}
      {/each}
    </div>

    <PresetIncludeToggles
      bind:includeGeometry
      bind:includeMasks
      bind:includeOutput
      bordered
    />

    <button
      class="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      disabled={busy || !presetId || selection.count === 0}
      onclick={() => void submit()}
    >
      {#if busy}
        <Icon path={mdiLoading} size={16} class="animate-spin" />
      {:else}
        <Icon path={mdiAutoFix} size={16} />
      {/if}
      {selected ? `Apply ${selected.name} to ${selection.count}` : `Select a preset`}
    </button>

    <p class="text-[11px] leading-relaxed text-immich-dark-fg/40">
      Runs as a background job. Track progress in the Jobs panel.
    </p>
  {/if}

  <div class="h-4"></div>
</div>
