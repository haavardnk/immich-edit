<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiUpload, mdiDelete, mdiRestore, mdiCheck, mdiClose, mdiAlertCircleOutline } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { listLuts, importLut, deleteLut, type LutMeta } from '$lib/api/luts';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';

  let luts = $state<LutMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state<string | null>(null);
  let fileInput: HTMLInputElement | null = $state(null);

  const selectedId = $derived(editor.edits.color.lut_3d.lut_id);
  const selected = $derived(luts.find((l) => l.id === selectedId) ?? null);
  const missingSelected = $derived(!!selectedId && !selected);

  $effect(() => {
    if (!loaded) void load();
  });

  async function load(): Promise<void> {
    try {
      luts = await listLuts();
    } catch {
      /* reported by client */
    }
    loaded = true;
  }

  function select(id: string | null): void {
    editor.edits.color.lut_3d.lut_id = id;
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
        if (existing && luts.some((l) => l.id === existing)) {
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

  async function confirmDelete(id: string): Promise<void> {
    try {
      await deleteLut(id);
    } catch {
      /* reported by client */
    }
    pendingDelete = null;
    if (selectedId === id) select(null);
    await load();
  }
</script>

<div class="flex flex-col gap-2.5 pb-1">
  <input
    bind:this={fileInput}
    type="file"
    accept=".cube"
    class="hidden"
    onchange={(e) => void onFile(e)}
  />
  <button
    class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={importing}
    onclick={triggerImport}
  >
    <Icon path={mdiUpload} size={14} />
    {importing ? 'Importing…' : 'Import .cube LUT'}
  </button>

  {#if missingSelected}
    <div class="flex items-center gap-1.5 rounded-lg bg-amber-500/10 px-2.5 py-1.5 text-[11px] text-amber-300">
      <Icon path={mdiAlertCircleOutline} size={14} />
      <span class="flex-1 min-w-0 truncate">Referenced LUT is unavailable</span>
      <button
        type="button"
        class="text-amber-300/70 hover:text-amber-200 transition-colors"
        title="Remove LUT"
        aria-label="Remove LUT"
        onclick={() => select(null)}
      >
        <Icon path={mdiClose} size={14} />
      </button>
    </div>
  {/if}

  {#if luts.length === 0}
    {#if loaded}
      <div class="text-xs text-immich-dark-fg/30 py-1">No LUTs imported yet.</div>
    {/if}
  {:else}
    <div class="flex flex-col gap-0.5">
      <button
        type="button"
        class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs text-left transition-colors {selectedId
          ? 'hover:bg-white/5 text-immich-dark-fg/60'
          : 'bg-white/10 text-immich-dark-fg'}"
        onclick={() => select(null)}
      >
        <span class="w-3.5 shrink-0">
          {#if !selectedId}<Icon path={mdiCheck} size={14} />{/if}
        </span>
        None
      </button>
      {#each luts as lut (lut.id)}
        <div
          class="group flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs transition-colors {selectedId ===
          lut.id
            ? 'bg-white/10 text-immich-dark-fg'
            : 'hover:bg-white/5 text-immich-dark-fg/70'}"
        >
          <button
            type="button"
            class="flex flex-1 min-w-0 items-center gap-1.5 text-left"
            title="{lut.lut_size}×{lut.lut_size}×{lut.lut_size} grid"
            onclick={() => select(lut.id)}
          >
            <span class="w-3.5 shrink-0">
              {#if selectedId === lut.id}<Icon path={mdiCheck} size={14} />{/if}
            </span>
            <span class="truncate">{lut.name}</span>
          </button>
          {#if pendingDelete === lut.id}
            <button
              type="button"
              class="text-red-400 hover:text-red-300 transition-colors"
              title="Confirm delete"
              aria-label="Confirm delete"
              onclick={() => void confirmDelete(lut.id)}
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
              title="Delete LUT"
              aria-label="Delete LUT"
              onclick={() => (pendingDelete = lut.id)}
            >
              <Icon path={mdiDelete} size={14} />
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if selected}
    <div class="flex flex-col gap-2.5 border-t border-white/10 pt-2">
      <div class="flex items-center justify-between px-1">
        <span class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60">Amount</span>
        <button
          type="button"
          class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
          title="Reset amount"
          aria-label="Reset amount"
          onclick={() => {
            editor.edits.color.lut_3d.amount = 100;
            editor.onCommit('LUT Amount');
          }}
        >
          <Icon path={mdiRestore} size={14} />
        </button>
      </div>
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
