<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import LutPicker from './lut/LutPicker.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiAlertCircleOutline,
    mdiCheck,
    mdiClose,
    mdiDelete,
    mdiRestore,
    mdiUpload
  } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listLuts, importLut, deleteLut, type LutMeta } from '$lib/api/luts';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';

  let luts = $state<LutMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state(false);
  let fileInput: HTMLInputElement | null = $state(null);

  const selectedId = $derived(editor.edits.color.lut_3d.lut_id);
  const selected = $derived(luts.find((lut) => lut.id === selectedId) ?? null);
  const missingSelected = $derived(loaded && !!selectedId && !selected);

  $effect(() => {
    if (!loaded) void load();
  });

  async function load(): Promise<void> {
    try {
      luts = await listLuts();
    } catch {
      luts = [];
    }
    loaded = true;
  }

  function select(id: string | null): void {
    editor.edits.color.lut_3d.lut_id = id;
    pendingDelete = false;
    editor.onCommit(id ? 'Select LUT' : 'Remove LUT');
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
      const meta = await importLut(name, bytes);
      await load();
      select(meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        if (existing && luts.some((lut) => lut.id === existing)) {
          select(existing);
          toasts.push('info', 'LUT already imported — selected existing.');
        }
      } else if (err instanceof ApiError && err.status === 400) {
        toasts.push('error', `Invalid LUT: ${err.message}`);
      } else {
        toasts.push('error', 'LUT import failed.');
      }
    } finally {
      importing = false;
    }
  }

  async function confirmDelete(): Promise<void> {
    if (!selected) return;
    try {
      await deleteLut(selected.id);
      select(null);
      await load();
    } catch {
      toasts.push('error', 'LUT delete failed.');
    }
    pendingDelete = false;
  }

  function reset(): void {
    editor.edits.color.lut_3d.lut_id = null;
    editor.edits.color.lut_3d.amount = 100;
    pendingDelete = false;
    editor.onCommit('Reset LUT');
  }
</script>

<div class="flex flex-col gap-2.5 pb-1">
  <div class="flex items-center justify-between">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">LUT</div>
    <button
      type="button"
      class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
      title="Reset LUT"
      aria-label="Reset LUT"
      onclick={reset}
    >
      <Icon path={mdiRestore} size={14} />
    </button>
  </div>
  <input
    bind:this={fileInput}
    type="file"
    accept=".cube"
    class="hidden"
    onchange={(e) => void onFile(e)}
  />

  <div class="flex items-center gap-1">
    <div class="min-w-0 flex-1">
      <LutPicker {luts} {selectedId} onSelect={select} />
    </div>
    {#if selected}
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
          title="Delete LUT"
          aria-label="Delete LUT"
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
    {importing ? 'Importing…' : 'Import .cube LUT'}
  </button>

  {#if missingSelected}
    <div
      class="flex items-center gap-1.5 rounded-lg bg-amber-500/10 px-2.5 py-1.5 text-[11px] text-amber-300"
    >
      <Icon path={mdiAlertCircleOutline} size={14} />
      <span class="min-w-0 flex-1 truncate">Referenced LUT is unavailable</span>
      <button
        type="button"
        class="text-amber-300/70 transition-colors hover:text-amber-200"
        title="Remove LUT"
        aria-label="Remove LUT"
        onclick={() => select(null)}
      >
        <Icon path={mdiClose} size={14} />
      </button>
    </div>
  {/if}

  {#if selected}
    <div class="border-t border-white/10 pt-2">
      <SliderRow
        label="Amount"
        commitAction="LUT Amount"
        bind:value={editor.edits.color.lut_3d.amount}
        min={0}
        max={100}
        step={1}
        defaultValue={100}
        onLive={editor.onLive}
        onCommit={editor.onCommit}
        format={(v: number) => v.toFixed(0)}
      />
    </div>
  {/if}
</div>
