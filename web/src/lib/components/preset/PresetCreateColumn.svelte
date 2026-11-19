<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiPlus, mdiUpload } from '@mdi/js';

  let { oncreate }: { oncreate: () => void } = $props();

  const buttonClass =
    'flex w-full items-center justify-center gap-1 rounded-sm border border-dashed border-light-950/20 text-[11px] text-dark/65 outline-none focus-visible:ring-2 focus-visible:ring-primary enabled:hover:border-light-950/40 enabled:hover:text-dark disabled:cursor-not-allowed disabled:opacity-50 touch:rounded-lg touch:text-[13px]';
</script>

<li class="flex w-24 flex-none flex-col gap-0.5 touch:min-h-28 touch:w-22 touch:gap-2">
  <button
    type="button"
    class="{buttonClass} min-h-0 flex-1 touch:flex-col touch:text-dark/80"
    title={editor.hasEdits ? 'Save current as preset' : 'Edit the photo to save a preset'}
    aria-label="Save current as preset"
    disabled={!editor.hasEdits}
    onclick={oncreate}
  >
    <Icon icon={mdiPlus} size="14px" class="touch:size-5" aria-hidden="true" />
    <span class="truncate">{editor.hasEdits ? 'New preset' : 'No edits yet'}</span>
  </button>
  <FileImportButton
    accept=".json,application/json"
    label="Import presets"
    onfile={presets.importFile}
  >
    {#snippet button({ busy, open })}
      <button
        type="button"
        class="{buttonClass} h-6 flex-none touch:h-9"
        title="Import presets from a file"
        aria-label="Import presets"
        disabled={busy}
        onclick={open}
      >
        <Icon icon={mdiUpload} size="14px" aria-hidden="true" />
        <span class="truncate">{busy ? 'Importing…' : 'Import'}</span>
      </button>
    {/snippet}
  </FileImportButton>
</li>
