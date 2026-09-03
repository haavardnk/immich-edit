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
  import Notice from '$lib/components/Notice.svelte';
  import { mdiDeleteOutline, mdiDownloadOutline } from '@mdi/js';
  import { Badge, IconButton, LoadingSpinner, ProgressBar, Select, Text } from '@immich/ui';

  let models = $state<MaskModelsResponse | null>(null);
  let error = $state<string | null>(null);
  let busyModels = $state<string[]>([]);

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

<section class="space-y-5 py-2">
  {#if error}
    <Notice message={error} />
  {/if}
  {#if !models}
    <div class="inline-flex items-center gap-2 text-dark/65" aria-live="polite">
      <LoadingSpinner size="small" />
      <span class="text-xs">Loading…</span>
    </div>
  {:else if !models.enabled}
    <Text size="tiny" color="muted">
      Segmentation is disabled. Set ML_RUNTIME to auto, gpu or cpu to enable it.
    </Text>
  {:else}
    <div class="flex min-h-8 items-center justify-between gap-3 text-xs text-dark/65">
      <span>
        Runtime <span class="font-mono text-dark">{models.runtime}</span> · {installedCount} of
        {models.models.length} installed
      </span>
      {#if missingRecommended.length > 0}
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          icon={mdiDownloadOutline}
          title={`Download ${missingRecommended.length} recommended (${formatMb(missingRecommended.reduce((total, model) => total + model.size_bytes, 0))})`}
          aria-label={`Download ${missingRecommended.length} recommended models`}
          onclick={() => void downloadRecommended()}
          disabled={busyModels.length > 0 || installing.length > 0}
        />
      {/if}
    </div>

    {#snippet modelRow(model: MaskModel)}
      <li class="flex min-h-18 items-center gap-3 py-2.5 text-xs">
        <div class="min-w-0 flex-1">
          <div class="flex min-w-0 flex-wrap items-center gap-1.5">
            <span class="truncate font-medium">{model.name}</span>
            {#if model.installed}<Badge size="tiny" color="success">installed</Badge>{/if}
            {#if model.tier === 'recommended'}
              <Badge size="tiny" color="primary">recommended</Badge>
            {/if}
            <Badge size="tiny" color="secondary">{model.license}</Badge>
          </div>
          <div class="mt-0.5 truncate text-dark/65">{model.notes}</div>
          <div class="truncate text-dark/65">
            {formatMb(model.size_bytes)} · {model.gpu_mb} MB VRAM · ~{formatSeconds(model.gpu_ms)} GPU{#if model.cpu_ms > 0}
              · ~{formatSeconds(model.cpu_ms)} CPU{/if}
          </div>
          {#if model.install_error}
            <div class="truncate text-danger-300">Download failed: {model.install_error}</div>
          {/if}
          {#if model.installing}
            <ProgressBar
              progress={installPercent(model)}
              max={100}
              size="tiny"
              shape="round"
              color="primary"
              class="mt-1"
              stop={false}
              aria-label={`${model.name} download progress`}
            />
          {/if}
        </div>
        <IconButton
          size="small"
          variant="ghost"
          color={model.installed ? 'danger' : 'secondary'}
          icon={model.installed ? mdiDeleteOutline : mdiDownloadOutline}
          title={model.installed
            ? 'Remove model'
            : model.install_error
              ? 'Retry download'
              : 'Download model'}
          aria-label={`${model.installed ? 'Remove' : model.install_error ? 'Retry' : 'Download'} ${model.name}`}
          onclick={() => void toggleModel(model)}
          disabled={busyModels.includes(model.id) || model.installing}
        />
      </li>
    {/snippet}

    <div class="space-y-6">
      {#each modelKinds as kind (kind)}
        {@const inKind = models.models.filter((model) => model.kind === kind)}
        {@const installed = inKind.filter((model) => model.installed)}
        <div>
          <div class="flex min-h-9 items-center justify-between gap-3 border-b border-hairline">
            <h2 class="text-xs font-medium">{kindLabel(kind)}</h2>
            {#if installed.length === 0}
              <span class="text-[11px] text-dark/45">Not installed</span>
            {:else}
              {@const modelOptions = installed.map((model) => ({
                value: model.id,
                label: model.name
              }))}
              <Select
                size="tiny"
                class="mask-model-select w-48"
                placeholder="Active model"
                options={modelOptions}
                value={models.active[kind] ?? installed[0]?.id}
                onChange={(value) => void chooseModel(kind, value)}
              />
            {/if}
          </div>
          <ul class="divide-y divide-hairline">
            {#each inKind as model (model.id)}
              {@render modelRow(model)}
            {/each}
          </ul>
        </div>
      {/each}
    </div>
  {/if}
</section>
