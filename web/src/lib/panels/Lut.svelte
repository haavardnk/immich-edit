<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { commitLut, commitSectionReset } from '#lib/components/editor/controls/paramReset.ts';
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import SearchableSelect from '#lib/components/SearchableSelect.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { editsToManifest } from '#lib/edits/manifest.ts';
  import { LUT, lutAmount } from '#lib/editor/params/color.ts';
  import { mdiClose } from '@mdi/js';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { listLuts, importLut, deleteLut, type LutMeta } from '#lib/api/luts.ts';
  import { ApiError } from '#lib/api/client.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { lutPreview } from '#lib/stores/lutPreview.svelte.ts';

  let luts = $state<LutMeta[]>([]);
  let loaded = $state(false);
  let pendingDelete = $state(false);

  const selectedId = $derived(editor.edits.color.lut_3d.lut_id);
  const selected = $derived(luts.find((lut) => lut.id === selectedId) ?? null);
  const missingSelected = $derived(loaded && !!selectedId && !selected);
  const modified = $derived('lut_3d' in editsToManifest(editor.edits).ops);

  $effect(() => {
    if (!loaded) void load();
  });

  $effect(() => lutPreview.end);

  function preview(lut: LutMeta | null): void {
    if (lut) lutPreview.show(lut);
    else lutPreview.end();
  }

  async function load(): Promise<void> {
    try {
      luts = await listLuts();
    } catch {
      luts = [];
    }
    loaded = true;
  }

  function select(id: string | null): void {
    pendingDelete = false;
    commitLut(id);
  }

  async function importFile(file: File): Promise<void> {
    const name = file.name.replace(/\.[^.]+$/, '');
    const bytes = new Uint8Array(await file.arrayBuffer());
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
    pendingDelete = false;
    commitSectionReset(LUT);
  }
</script>

<div class="flex flex-col gap-1.5 pb-1">
  <SectionHeader title="LUT" section="lut" {modified} onReset={reset} />

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
        getDescription={(lut) =>
          [lut.shaper_size && `1D ${lut.shaper_size}`, lut.lut_size && `${lut.lut_size}³`]
            .filter(Boolean)
            .join(' + ')}
        placeholder="Select LUT…"
        hideSelected={false}
        previewing={lutPreview.id}
        onSelectedChange={(ids) => select(ids.at(-1) ?? null)}
        onPreview={preview}
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
    <FileImportButton accept=".cube" label="Import .cube LUT" onfile={importFile} />
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
      <ParamSlider param={lutAmount} />
    </div>
  {/if}
</div>
