<script lang="ts">
  import DcpPicker from './dcp/DcpPicker.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiAlertCircleOutline,
    mdiCheck,
    mdiChevronDown,
    mdiClose,
    mdiDelete,
    mdiTuneVariant,
    mdiUpload
  } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listDcps, matchDcp, importDcp, deleteDcp, type DcpMeta } from '$lib/api/dcp';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { DcpIlluminant, DcpMode } from '$lib/types/edits';

  let dcps = $state<DcpMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let optionsOpen = $state(false);
  let pendingDelete = $state(false);
  let autoMatch = $state<DcpMeta | null>(null);
  let fileInput: HTMLInputElement | null = $state(null);

  const dcp = $derived(editor.edits.color.dcp);
  const selectedId = $derived(dcp.mode === 'profile' ? dcp.profile_id : null);
  const selected = $derived(dcps.find((profile) => profile.id === selectedId) ?? null);
  const missingSelected = $derived(loaded && !!selectedId && !selected);
  const cameraModel = $derived(editor.asset?.exifInfo?.model ?? null);
  const cameraMake = $derived(editor.asset?.exifInfo?.make ?? null);

  const illuminants: { value: DcpIlluminant; label: string }[] = [
    { value: 'interpolated', label: 'Auto' },
    { value: 'first', label: 'Illum 1' },
    { value: 'second', label: 'Illum 2' }
  ];

  $effect(() => {
    if (!loaded) void load();
  });

  $effect(() => {
    const model = cameraModel;
    const make = cameraMake;
    if (!model) {
      autoMatch = null;
    } else {
      void loadAutoMatch(model, make);
    }
  });

  async function load(): Promise<void> {
    try {
      dcps = await listDcps();
    } catch {
      dcps = [];
    }
    loaded = true;
  }

  async function loadAutoMatch(model: string, make: string | null): Promise<void> {
    try {
      autoMatch = await matchDcp(model, make);
    } catch {
      autoMatch = null;
    }
  }

  function select(mode: DcpMode, id: string | null): void {
    dcp.mode = mode;
    dcp.profile_id = id;
    pendingDelete = false;
    const action =
      mode === 'auto' ? 'DCP Auto Match' : mode === 'off' ? 'Disable DCP' : 'Select DCP Profile';
    editor.onCommit(action);
  }

  function setIlluminant(value: DcpIlluminant): void {
    dcp.illuminant = value;
    editor.onCommit('DCP Illuminant');
  }

  function toggle(
    field: 'use_tone_curve' | 'use_base_table' | 'use_look_table' | 'use_baseline_exposure',
    label: string
  ): void {
    dcp[field] = !dcp[field];
    editor.onCommit(label);
  }

  function triggerImport(): void {
    fileInput?.click();
  }

  async function onFile(e: Event): Promise<void> {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    const name = file.name.replace(/\.[^.]+$/, '');
    const bytes = new Uint8Array(await file.arrayBuffer());
    importing = true;
    try {
      const meta = await importDcp(name, bytes);
      await load();
      select('profile', meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        const duplicate = dcps.find((profile) => profile.id === existing);
        if (duplicate) {
          select(duplicate.bundled ? 'auto' : 'profile', duplicate.bundled ? null : duplicate.id);
          toasts.push(
            'info',
            duplicate.bundled
              ? 'Profile is already included in Auto match.'
              : 'Profile already imported — selected existing.'
          );
        }
      } else if (err instanceof ApiError && err.status === 400) {
        toasts.push('error', `Invalid profile: ${err.message}`);
      } else {
        toasts.push('error', 'Profile import failed.');
      }
    } finally {
      importing = false;
    }
  }

  async function confirmDelete(): Promise<void> {
    if (!selected || selected.bundled) return;
    try {
      await deleteDcp(selected.id);
      select('auto', null);
      await load();
    } catch {
      toasts.push('error', 'Profile delete failed.');
    }
    pendingDelete = false;
  }
</script>

<div class="flex flex-col gap-2.5 pb-1">
  <input
    bind:this={fileInput}
    type="file"
    accept=".dcp"
    class="hidden"
    onchange={(e) => void onFile(e)}
  />

  <div class="flex items-center gap-1">
    <div class="min-w-0 flex-1">
      <DcpPicker
        profiles={dcps}
        mode={dcp.mode}
        {selectedId}
        {autoMatch}
        {cameraModel}
        onSelect={select}
      />
    </div>
    {#if selected && !selected.bundled}
      {#if pendingDelete}
        <button
          type="button"
          class="flex items-center justify-center rounded-lg p-1.5 text-red-400 transition-colors hover:bg-white/10 hover:text-red-300"
          title="Confirm delete"
          aria-label="Confirm delete"
          onclick={() => void confirmDelete()}
        >
          <Icon path={mdiCheck} size={14} />
        </button>
        <button
          type="button"
          class="flex items-center justify-center rounded-lg p-1.5 text-immich-dark-fg/40 transition-colors hover:bg-white/10 hover:text-immich-dark-fg"
          title="Cancel"
          aria-label="Cancel delete"
          onclick={() => (pendingDelete = false)}
        >
          <Icon path={mdiClose} size={14} />
        </button>
      {:else}
        <button
          type="button"
          class="flex items-center justify-center rounded-lg p-1.5 text-immich-dark-fg/40 transition-colors hover:bg-white/10 hover:text-red-400"
          title="Delete imported profile"
          aria-label="Delete imported profile"
          onclick={() => (pendingDelete = true)}
        >
          <Icon path={mdiDelete} size={14} />
        </button>
      {/if}
    {/if}
  </div>

  <button
    type="button"
    class="flex items-center justify-center gap-1.5 rounded-lg bg-white/5 py-1.5 text-xs transition-colors hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-40"
    disabled={importing}
    onclick={triggerImport}
  >
    <Icon path={mdiUpload} size={14} />
    {importing ? 'Importing…' : 'Import .dcp profile'}
  </button>

  {#if missingSelected}
    <div
      class="flex items-center gap-1.5 rounded-lg bg-amber-500/10 px-2.5 py-1.5 text-[11px] text-amber-300"
    >
      <Icon path={mdiAlertCircleOutline} size={14} />
      <span class="min-w-0 flex-1 truncate">Referenced profile is unavailable</span>
      <button
        type="button"
        class="text-amber-300/70 transition-colors hover:text-amber-200"
        title="Return to auto match"
        aria-label="Return to auto match"
        onclick={() => select('auto', null)}
      >
        <Icon path={mdiClose} size={14} />
      </button>
    </div>
  {/if}

  {#if dcp.mode !== 'off'}
    <button
      type="button"
      class="flex items-center gap-1.5 px-1 text-[10px] uppercase tracking-wide text-immich-dark-fg/50 transition-colors hover:text-immich-dark-fg/80"
      onclick={() => (optionsOpen = !optionsOpen)}
    >
      <Icon path={mdiTuneVariant} size={13} />
      <span class="flex-1 text-left">Profile options</span>
      <Icon
        path={mdiChevronDown}
        size={13}
        class="transition-transform {optionsOpen ? 'rotate-180' : ''}"
      />
    </button>

    {#if optionsOpen}
      <div class="flex flex-col gap-2.5 border-t border-white/10 pt-2.5">
        <div class="flex flex-col gap-1">
          <span class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60">Illuminant</span>
          <div class="flex gap-0.5 rounded-lg bg-white/5 p-0.5">
            {#each illuminants as illuminant (illuminant.value)}
              <button
                type="button"
                class="flex-1 rounded-md px-2 py-1 text-[11px] transition-colors {dcp.illuminant ===
                illuminant.value
                  ? 'bg-white/15 text-immich-dark-fg'
                  : 'text-immich-dark-fg/60 hover:text-immich-dark-fg'}"
                onclick={() => setIlluminant(illuminant.value)}
              >
                {illuminant.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="flex flex-col gap-1.5">
          <label class="flex cursor-pointer items-center gap-2 text-[11px] text-immich-dark-fg/80">
            <input
              type="checkbox"
              class="checkbox checkbox-xs checkbox-primary"
              checked={dcp.use_base_table}
              onchange={() => toggle('use_base_table', 'DCP Base Table')}
            />
            Base table (HueSatMap)
          </label>
          <label class="flex cursor-pointer items-center gap-2 text-[11px] text-immich-dark-fg/80">
            <input
              type="checkbox"
              class="checkbox checkbox-xs checkbox-primary"
              checked={dcp.use_tone_curve}
              onchange={() => toggle('use_tone_curve', 'DCP Tone Curve')}
            />
            Tone curve
          </label>
          <label class="flex cursor-pointer items-center gap-2 text-[11px] text-immich-dark-fg/80">
            <input
              type="checkbox"
              class="checkbox checkbox-xs checkbox-primary"
              checked={dcp.use_look_table}
              onchange={() => toggle('use_look_table', 'DCP Look Table')}
            />
            Look table
          </label>
          <label class="flex cursor-pointer items-center gap-2 text-[11px] text-immich-dark-fg/80">
            <input
              type="checkbox"
              class="checkbox checkbox-xs checkbox-primary"
              checked={dcp.use_baseline_exposure}
              onchange={() => toggle('use_baseline_exposure', 'DCP Baseline Exposure')}
            />
            Baseline exposure
          </label>
        </div>
      </div>
    {/if}
  {/if}
</div>
