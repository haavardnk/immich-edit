<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiUpload,
    mdiDelete,
    mdiCheck,
    mdiClose,
    mdiAlertCircleOutline,
    mdiMagnify
  } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listDcps, importDcp, deleteDcp, type DcpMeta } from '$lib/api/dcp';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';
  import type { DcpIlluminant } from '$lib/types/edits';

  let dcps = $state<DcpMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state<string | null>(null);
  let query = $state('');
  let fileInput: HTMLInputElement | null = $state(null);

  const dcp = $derived(editor.edits.color.dcp);
  const selectedId = $derived(dcp.mode === 'profile' ? dcp.profile_id : null);
  const selected = $derived(dcps.find((d) => d.id === selectedId) ?? null);
  const missingSelected = $derived(!!selectedId && !selected);
  const filtered = $derived(
    query.trim()
      ? dcps.filter((d) => {
          const q = query.trim().toLowerCase();
          return (
            d.name.toLowerCase().includes(q) ||
            (d.camera_model?.toLowerCase().includes(q) ?? false)
          );
        })
      : dcps
  );

  const illuminants: { value: DcpIlluminant; label: string }[] = [
    { value: 'interpolated', label: 'Auto' },
    { value: 'first', label: 'Illum 1' },
    { value: 'second', label: 'Illum 2' }
  ];

  $effect(() => {
    if (!loaded) void load();
  });

  async function load(): Promise<void> {
    try {
      dcps = await listDcps();
    } catch {
      /* reported by client */
    }
    loaded = true;
  }

  function selectNone(): void {
    dcp.mode = 'off';
    dcp.profile_id = null;
    editor.onCommit('Remove DCP Profile');
  }

  function selectAuto(): void {
    dcp.mode = 'auto';
    dcp.profile_id = null;
    editor.onCommit('DCP Auto Match');
  }

  function selectProfile(id: string): void {
    dcp.mode = 'profile';
    dcp.profile_id = id;
    editor.onCommit('Select DCP Profile');
  }

  function setIlluminant(value: DcpIlluminant): void {
    dcp.illuminant = value;
    editor.onCommit('DCP Illuminant');
  }

  function toggle(field: 'use_tone_curve' | 'use_base_table' | 'use_look_table' | 'use_baseline_exposure', label: string): void {
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
      selectProfile(meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        if (existing && dcps.some((d) => d.id === existing)) {
          selectProfile(existing);
          toasts.push('info', 'Profile already imported — selected existing.');
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

  async function confirmDelete(id: string): Promise<void> {
    try {
      await deleteDcp(id);
    } catch {
      /* reported by client */
    }
    pendingDelete = null;
    if (selectedId === id) selectNone();
    await load();
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
  <button
    class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={importing}
    onclick={triggerImport}
  >
    <Icon path={mdiUpload} size={14} />
    {importing ? 'Importing…' : 'Import .dcp profile'}
  </button>

  {#if missingSelected}
    <div class="flex items-center gap-1.5 rounded-lg bg-amber-500/10 px-2.5 py-1.5 text-[11px] text-amber-300">
      <Icon path={mdiAlertCircleOutline} size={14} />
      <span class="flex-1 min-w-0 truncate">Referenced profile is unavailable</span>
      <button
        type="button"
        class="text-amber-300/70 hover:text-amber-200 transition-colors"
        title="Remove profile"
        aria-label="Remove profile"
        onclick={selectNone}
      >
        <Icon path={mdiClose} size={14} />
      </button>
    </div>
  {/if}

  <div class="flex flex-col gap-0.5">
    <button
      type="button"
      class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs text-left transition-colors {dcp.mode ===
      'off'
        ? 'bg-white/10 text-immich-dark-fg'
        : 'hover:bg-white/5 text-immich-dark-fg/60'}"
      onclick={selectNone}
    >
      <span class="w-3.5 shrink-0">
        {#if dcp.mode === 'off'}<Icon path={mdiCheck} size={14} />{/if}
      </span>
      None
    </button>
    <button
      type="button"
      class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs text-left transition-colors {dcp.mode ===
      'auto'
        ? 'bg-white/10 text-immich-dark-fg'
        : 'hover:bg-white/5 text-immich-dark-fg/70'}"
      onclick={selectAuto}
    >
      <span class="w-3.5 shrink-0">
        {#if dcp.mode === 'auto'}<Icon path={mdiCheck} size={14} />{/if}
      </span>
      Auto (match camera)
    </button>
  </div>

  {#if dcps.length > 8}
    <div class="flex items-center gap-1.5 rounded-lg bg-white/5 px-2.5 py-1">
      <Icon path={mdiMagnify} size={14} />
      <input
        type="text"
        bind:value={query}
        placeholder="Search profiles…"
        class="flex-1 min-w-0 bg-transparent text-xs outline-none placeholder:text-immich-dark-fg/30"
      />
    </div>
  {/if}

  {#if loaded && dcps.length === 0}
    <div class="text-xs text-immich-dark-fg/30 py-1">No profiles available.</div>
  {:else if filtered.length > 0}
    <div class="flex max-h-64 flex-col gap-0.5 overflow-y-auto">
      {#each filtered as profile (profile.id)}
        <div
          class="group flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs transition-colors {selectedId ===
          profile.id
            ? 'bg-white/10 text-immich-dark-fg'
            : 'hover:bg-white/5 text-immich-dark-fg/70'}"
        >
          <button
            type="button"
            class="flex flex-1 min-w-0 items-center gap-1.5 text-left"
            title={profile.camera_model ?? profile.name}
            onclick={() => selectProfile(profile.id)}
          >
            <span class="w-3.5 shrink-0">
              {#if selectedId === profile.id}<Icon path={mdiCheck} size={14} />{/if}
            </span>
            <span class="truncate">{profile.camera_model ?? profile.name}</span>
          </button>
          {#if !profile.bundled}
            {#if pendingDelete === profile.id}
              <button
                type="button"
                class="text-red-400 hover:text-red-300 transition-colors"
                title="Confirm delete"
                aria-label="Confirm delete"
                onclick={() => void confirmDelete(profile.id)}
              >
                <Icon path={mdiCheck} size={14} />
              </button>
              <button
                type="button"
                class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
                title="Cancel"
                aria-label="Cancel"
                onclick={() => (pendingDelete = null)}
              >
                <Icon path={mdiClose} size={14} />
              </button>
            {:else}
              <button
                type="button"
                class="text-immich-dark-fg/0 group-hover:text-immich-dark-fg/40 hover:!text-red-400 transition-colors"
                title="Delete profile"
                aria-label="Delete profile"
                onclick={() => (pendingDelete = profile.id)}
              >
                <Icon path={mdiDelete} size={14} />
              </button>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
  {:else if loaded}
    <div class="text-xs text-immich-dark-fg/30 py-1">No matching profiles.</div>
  {/if}

  {#if dcp.mode !== 'off'}
    <div class="flex flex-col gap-2.5 border-t border-white/10 pt-2.5">
      <div class="flex flex-col gap-1">
        <span class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60">Illuminant</span>
        <div class="flex gap-0.5 rounded-lg bg-white/5 p-0.5">
          {#each illuminants as ill (ill.value)}
            <button
              type="button"
              class="flex-1 rounded-md px-2 py-1 text-[11px] transition-colors {dcp.illuminant ===
              ill.value
                ? 'bg-white/15 text-immich-dark-fg'
                : 'text-immich-dark-fg/60 hover:text-immich-dark-fg'}"
              onclick={() => setIlluminant(ill.value)}
            >
              {ill.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
          <input
            type="checkbox"
            class="checkbox checkbox-xs checkbox-primary"
            checked={dcp.use_base_table}
            onchange={() => toggle('use_base_table', 'DCP Base Table')}
          />
          Base table (HueSatMap)
        </label>
        <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
          <input
            type="checkbox"
            class="checkbox checkbox-xs checkbox-primary"
            checked={dcp.use_tone_curve}
            onchange={() => toggle('use_tone_curve', 'DCP Tone Curve')}
          />
          Tone curve
        </label>
        <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
          <input
            type="checkbox"
            class="checkbox checkbox-xs checkbox-primary"
            checked={dcp.use_look_table}
            onchange={() => toggle('use_look_table', 'DCP Look Table')}
          />
          Look table
        </label>
        <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
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
</div>
