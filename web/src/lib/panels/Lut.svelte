<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import DeleteConfirmation from '$lib/components/DeleteConfirmation.svelte';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { editsToManifest } from '$lib/types/edits';
  import { mdiClose, mdiUpload } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { session } from '$lib/stores/session.svelte';
  import { listLuts, importLut, deleteLut, type LutMeta } from '$lib/api/luts';
  import { ApiError } from '$lib/api/client';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { Button, IconButton } from '@immich/ui';

  let luts = $state<LutMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state(false);
  let fileInput: HTMLInputElement | null = $state(null);

  const selectedId = $derived(editor.edits.color.lut_3d.lut_id);
  const selected = $derived(luts.find((lut) => lut.id === selectedId) ?? null);
  const missingSelected = $derived(loaded && !!selectedId && !selected);
  const modified = $derived('lut_3d' in editsToManifest(editor.edits).ops);

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
  }

  function reset(): void {
    editor.edits.color.lut_3d.lut_id = null;
    editor.edits.color.lut_3d.amount = 100;
    pendingDelete = false;
    editor.onCommit('Reset LUT');
  }
</script>

<div class="flex flex-col gap-1.5 pb-1">
  <SectionHeader title="LUT" {modified} onReset={reset} />
  <input
    bind:this={fileInput}
    type="file"
    accept=".cube"
    class="hidden"
    onchange={(e) => void onFile(e)}
  />

  <div class="flex items-center gap-1">
    <div class="min-w-0 flex-1">
      <SearchableSelect
        compact
        multiple={false}
        color="neutral"
        options={luts}
        selected={selectedId ? [selectedId] : []}
        getId={(lut) => lut.id}
        getLabel={(lut) => lut.name}
        getDescription={(lut) => `${lut.lut_size}³`}
        placeholder="Select LUT…"
        hideSelected={false}
        onSelectedChange={(ids) => select(ids.at(-1) ?? null)}
      />
    </div>
    {#if selected && session.isAdmin}
      <DeleteConfirmation
        bind:pending={pendingDelete}
        label="Delete LUT"
        confirmLabel="Confirm delete LUT"
        onconfirm={confirmDelete}
      />
    {/if}
  </div>

  {#if session.isAdmin}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-7 panel-action"
      leadingIcon={mdiUpload}
      disabled={importing}
      onclick={triggerImport}
    >
      {importing ? 'Importing…' : 'Import .cube LUT'}
    </Button>
  {/if}

  {#if missingSelected}
    <Notice color="warning" message="Referenced LUT is unavailable" class="mx-1">
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        title="Remove LUT"
        aria-label="Remove LUT"
        onclick={() => select(null)}
      />
    </Notice>
  {/if}

  {#if selected}
    <div class="border-t border-hairline pt-1">
      <EditSlider
        label="Amount"
        commitAction="LUT Amount"
        bind:value={editor.edits.color.lut_3d.amount}
        min={0}
        max={100}
        defaultValue={100}
      />
    </div>
  {/if}
</div>
