<script lang="ts">
  import {
    cancelModelInstall,
    installModel,
    listModels,
    removeModel,
    selectDefaultModel,
    type Model,
    type ModelKind,
    type ModelsResponse
  } from '#lib/api/models.ts';
  import Notice from '#lib/components/Notice.svelte';
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { formatMb, formatSeconds, installPercent, kindLabel } from '#lib/utils/models.ts';
  import { Badge, Button, IconButton, ProgressBar, Select } from '@immich/ui';
  import { mdiClose, mdiDeleteOutline, mdiDownloadOutline } from '@mdi/js';
  import { onMount } from 'svelte';

  let models = $state<ModelsResponse | null>(null);
  let error = $state<string | null>(null);
  let busyModels = $state<string[]>([]);

  const modelKinds = $derived([...new Set((models?.models ?? []).map((m) => m.kind))]);
  const missingRecommended = $derived(
    (models?.models ?? []).filter((m) => !m.installed && !m.installing && m.tier === 'recommended')
  );
  const installing = $derived((models?.models ?? []).filter((m) => m.installing));
  const recommendedLabel = $derived(
    `Download ${missingRecommended.length} recommended (${formatMb(missingRecommended.reduce((total, model) => total + model.size_bytes, 0))})`
  );

  $effect(() => {
    if (installing.length === 0) return;
    const timer = setInterval(() => void load(), 1000);
    return () => clearInterval(timer);
  });

  async function load(): Promise<void> {
    try {
      models = await listModels();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function toggleModel(m: Model): Promise<void> {
    if (busyModels.includes(m.id)) return;
    busyModels = [...busyModels, m.id];
    error = null;
    try {
      if (m.installing) {
        await cancelModelInstall(m.id);
      } else if (m.installed) {
        await removeModel(m.id);
      } else {
        await installModel(m.id);
      }
      await load();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      busyModels = busyModels.filter((id) => id !== m.id);
    }
  }

  async function downloadRecommended(): Promise<void> {
    error = null;
    await Promise.all(missingRecommended.map((m) => toggleModel(m)));
  }

  async function chooseModel(kind: ModelKind, modelId: string): Promise<void> {
    error = null;
    try {
      await selectDefaultModel(kind, modelId);
      await load();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(() => {
    void load();
  });
</script>

{#snippet recommendedAction()}
  {#if models?.enabled && missingRecommended.length > 0}
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      class="touch:min-h-11"
      leadingIcon={mdiDownloadOutline}
      aria-label={`Download ${missingRecommended.length} recommended models`}
      onclick={() => void downloadRecommended()}
      disabled={busyModels.length > 0 || installing.length > 0}
    >
      {recommendedLabel}
    </Button>
  {/if}
{/snippet}

{#snippet modelRow(model: Model)}
  {@const action = model.installing
    ? 'Cancel download'
    : model.installed
      ? 'Remove'
      : model.install_error
        ? 'Retry'
        : 'Download'}
  <SettingsRow label={model.name}>
    {#snippet badge()}
      {#if model.tier === 'recommended'}<Badge size="tiny" color="primary">recommended</Badge>{/if}
    {/snippet}
    {#snippet detail()}
      {formatMb(model.size_bytes)} · {model.gpu_mb} MB VRAM · ~{formatSeconds(model.gpu_ms)} GPU{#if model.cpu_ms > 0}
        · ~{formatSeconds(model.cpu_ms)} CPU{/if}{#if model.kind === 'denoise'}
        per {model.input_edge} px tile{/if} · {model.license}
      {#if model.install_error}
        <span class="block text-danger-800">Download failed: {model.install_error}</span>
      {/if}
      {#if model.installing}
        <ProgressBar
          progress={installPercent(model)}
          max={100}
          size="tiny"
          shape="round"
          color="primary"
          class="mt-1.5"
          stop={false}
          aria-label={`${model.name} download progress`}
        />
      {/if}
    {/snippet}
    {#if model.installed}
      <IconButton
        size="small"
        variant="ghost"
        color="danger"
        class="shrink-0 touch:size-11"
        icon={mdiDeleteOutline}
        title="Remove model"
        aria-label={`${action} ${model.name}`}
        onclick={() => void toggleModel(model)}
        disabled={busyModels.includes(model.id)}
      />
    {:else}
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="shrink-0 bg-light-200/60 touch:min-h-11"
        leadingIcon={model.installing ? mdiClose : mdiDownloadOutline}
        aria-label={`${action} ${model.name}`}
        onclick={() => void toggleModel(model)}
        disabled={busyModels.includes(model.id)}
      >
        {model.installing ? 'Cancel' : model.install_error ? 'Retry' : 'Download'}
      </Button>
    {/if}
  </SettingsRow>
{/snippet}

<SettingsGroup
  title="Models"
  note={models?.enabled ? `Runtime ${models.runtime}` : undefined}
  anchor="models"
  action={recommendedAction}
>
  {#if error}
    <div class="px-4 py-3"><Notice message={error} /></div>
  {/if}
  {#if !models}
    <SettingsRow label="Loading…" muted />
  {:else if !models.enabled}
    <SettingsRow
      label="AI models are disabled"
      hint="Set ML_RUNTIME to auto, gpu or cpu to enable them."
      muted
    />
  {:else}
    {#each modelKinds as kind (kind)}
      {@const inKind = models.models.filter((model) => model.kind === kind)}
      {@const installed = inKind.filter((model) => model.installed)}
      <div
        class="flex min-h-10 items-center gap-3 bg-light-50/60 px-4 text-caption font-medium tracking-wide text-dark/80 uppercase touch:text-xs"
      >
        <h3 class="min-w-0 flex-1 truncate">{kindLabel(kind)}</h3>
        {#if installed.length > 0}
          <Select
            size="tiny"
            class="model-select w-48 normal-case"
            placeholder="Active model"
            options={installed.map((model) => ({ value: model.id, label: model.name }))}
            value={models.active[kind] ?? installed[0]?.id}
            onChange={(value) => void chooseModel(kind, value)}
          />
        {/if}
      </div>
      {#each inKind as model (model.id)}
        {@render modelRow(model)}
      {/each}
    {/each}
  {/if}
</SettingsGroup>
