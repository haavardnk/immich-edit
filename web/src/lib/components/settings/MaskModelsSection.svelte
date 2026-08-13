<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listMaskModels,
    installMaskModel,
    removeMaskModel,
    selectMaskModel,
    type MaskKind,
    type MaskModel,
    type MaskModelsResponse
  } from '$lib/api/masks';
  import { kindLabel, installPercent, formatSeconds, formatMb } from '$lib/utils/maskModels';
  import Spinner from '$lib/components/Spinner.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let models = $state<MaskModelsResponse | null>(null);
  let error = $state<string | null>(null);
  let busyModels = $state<string[]>([]);
  let open = $state(false);
  let expandedKinds = $state<string[]>([]);

  const modelKinds = $derived([...new Set((models?.models ?? []).map((m) => m.kind))]);
  const missingRecommended = $derived(
    (models?.models ?? []).filter((m) => !m.installed && !m.installing && m.tier === 'recommended')
  );
  const installedCount = $derived((models?.models ?? []).filter((m) => m.installed).length);
  const installing = $derived((models?.models ?? []).filter((m) => m.installing));

  $effect(() => {
    if (installing.length === 0) return;
    const timer = setInterval(() => void load(), 1000);
    return () => clearInterval(timer);
  });

  function toggleKind(kind: string): void {
    expandedKinds = expandedKinds.includes(kind)
      ? expandedKinds.filter((k) => k !== kind)
      : [...expandedKinds, kind];
  }

  async function load(): Promise<void> {
    try {
      models = await listMaskModels();
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function toggleModel(m: MaskModel): Promise<void> {
    if (busyModels.includes(m.id) || m.installing) return;
    busyModels = [...busyModels, m.id];
    error = null;
    try {
      if (m.installed) {
        await removeMaskModel(m.id);
      } else {
        await installMaskModel(m.id);
      }
      await load();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busyModels = busyModels.filter((id) => id !== m.id);
    }
  }

  async function downloadRecommended(): Promise<void> {
    error = null;
    await Promise.all(missingRecommended.map((m) => toggleModel(m)));
  }

  async function chooseModel(kind: MaskKind, modelId: string): Promise<void> {
    error = null;
    try {
      await selectMaskModel(kind, modelId);
      await load();
    } catch (e) {
      error = (e as Error).message;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<section class="space-y-2">
  <button
    class="-mx-2 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left transition-colors hover:bg-white/5"
    onclick={() => (open = !open)}
    aria-expanded={open}
  >
    <Icon
      path={open ? mdiChevronDown : mdiChevronRight}
      size={16}
      class="flex-none text-immich-dark-fg/50"
    />
    <h2 class="text-xs uppercase tracking-wider text-immich-dark-fg/50">Mask models</h2>
    <span class="ml-auto text-xs text-immich-dark-fg/40">
      {#if installing.length > 0}
        Downloading {installPercent(installing[0])}%
      {:else if models}
        {installedCount} of {models.models.length} installed
      {/if}
    </span>
  </button>
  {#if error}
    <p class="text-xs text-red-300">{error}</p>
  {/if}
  {#if open}
    {#if !models}
      <Spinner label="Loading…" />
    {:else if !models.enabled}
      <p class="text-xs text-immich-dark-fg/50">
        Segmentation is disabled. Set ML_RUNTIME to auto, gpu or cpu to enable it.
      </p>
    {:else}
      <p class="text-xs text-immich-dark-fg/50">
        Runtime: <span class="font-mono">{models.runtime}</span>. Models are downloaded on demand
        and loaded only while a mask is being generated.
      </p>

      {#if missingRecommended.length > 0}
        <button
          class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-xs disabled:opacity-50"
          onclick={() => void downloadRecommended()}
          disabled={busyModels.length > 0 || installing.length > 0}
        >
          Download {missingRecommended.length} recommended ({formatMb(
            missingRecommended.reduce((n, m) => n + m.size_bytes, 0)
          )})
        </button>
      {/if}

      {#snippet modelRow(m: MaskModel)}
        <li class="flex items-center justify-between rounded bg-white/5 px-3 py-2 text-xs">
          <div class="min-w-0 flex-1">
            <div class="truncate">
              {m.name}
              {#if m.tier === 'recommended'}<span class="text-immich-primary">
                  · recommended</span
                >{/if}
              <span class="text-immich-dark-fg/40"> · {m.license}</span>
            </div>
            <div class="text-immich-dark-fg/40 truncate">{m.notes}</div>
            <div class="text-immich-dark-fg/40 truncate">
              {formatMb(m.size_bytes)} download · ~{m.gpu_mb} MB while running · ~{formatSeconds(
                m.gpu_ms
              )} per photo on GPU{#if m.cpu_ms > 0}, ~{formatSeconds(m.cpu_ms)} on CPU{/if}
            </div>
            {#if m.install_error}
              <div class="text-red-300 truncate">Download failed: {m.install_error}</div>
            {/if}
          </div>
          <div class="flex items-center gap-2 shrink-0 ml-2">
            <button
              class="relative w-24 overflow-hidden px-2 py-1 rounded text-center disabled:opacity-50 {m.installed
                ? 'bg-red-500/10 text-red-300 hover:bg-red-500/20'
                : 'bg-white/5 hover:bg-white/10'}"
              onclick={() => void toggleModel(m)}
              disabled={busyModels.includes(m.id) || m.installing}
            >
              {#if m.installing}
                <span
                  class="absolute inset-y-0 left-0 bg-immich-primary/30"
                  style:width="{installPercent(m)}%"
                ></span>
              {/if}
              <span class="relative">
                {#if m.installing}
                  {installPercent(m)}%
                {:else if busyModels.includes(m.id)}
                  {m.installed ? 'Removing…' : 'Starting…'}
                {:else if m.install_error}
                  Retry
                {:else}
                  {m.installed ? 'Remove' : 'Download'}
                {/if}
              </span>
            </button>
          </div>
        </li>
      {/snippet}

      <div class="space-y-3">
        {#each modelKinds as kind (kind)}
          {@const inKind = models.models.filter((m) => m.kind === kind)}
          {@const installed = inKind.filter((m) => m.installed)}
          {@const primary =
            installed.length > 0 ? installed : inKind.filter((m) => m.tier === 'recommended')}
          {@const alternatives = inKind.filter((m) => !primary.includes(m))}
          {@const kindOpen = expandedKinds.includes(kind)}
          <div class="space-y-1">
            <div class="flex items-center justify-between gap-2 text-xs">
              <span class="text-immich-dark-fg/50">{kindLabel(kind)} masks use</span>
              {#if installed.length === 0}
                <span class="text-immich-dark-fg/40">nothing installed</span>
              {:else}
                <select
                  class="rounded bg-black/30 border border-white/10 px-2 py-1"
                  value={models.active[kind] ?? installed[0].id}
                  onchange={(e) => void chooseModel(kind, e.currentTarget.value)}
                >
                  {#each installed as m (m.id)}
                    <option value={m.id}>{m.name}</option>
                  {/each}
                </select>
              {/if}
            </div>
            <ul class="space-y-1">
              {#each primary as m (m.id)}
                {@render modelRow(m)}
              {/each}
              {#if kindOpen}
                {#each alternatives as m (m.id)}
                  {@render modelRow(m)}
                {/each}
              {/if}
            </ul>
            {#if alternatives.length > 0}
              <button
                class="text-xs text-immich-dark-fg/50 hover:text-immich-dark-fg/80"
                onclick={() => toggleKind(kind)}
              >
                {kindOpen ? 'Hide' : 'Show'}
                {alternatives.length} alternative{alternatives.length === 1 ? '' : 's'}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>
