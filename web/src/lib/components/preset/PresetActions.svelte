<script lang="ts">
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import MenuItem from '#lib/components/MenuItem.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal, mdiDownloadMultiple, mdiPlus, mdiUpload } from '@mdi/js';

  let { oncreate }: { oncreate: () => void } = $props();

  let menuOpen = $state(false);
  const size = 'small';

  function act(action: () => void): void {
    menuOpen = false;
    action();
  }
</script>

<DisabledReason locked={!editor.hasEdits} reason="Edit the photo to save a preset">
  <IconButton
    {size}
    variant="ghost"
    color="secondary"
    icon={mdiPlus}
    title={editor.hasEdits ? 'Save current as preset' : ''}
    aria-label="Save current as preset"
    disabled={!editor.hasEdits}
    onclick={oncreate}
  />
</DisabledReason>
<FileImportButton
  accept=".json,application/json"
  label="Import presets"
  onfile={presets.importFile}
>
  {#snippet button({ busy, open })}
    <Popover
      open={menuOpen}
      anchor="bottom"
      align="end"
      onOpenChange={(next) => (menuOpen = next)}
      contentClass="w-52 py-1"
      title="Presets"
      tooltip="Preset actions"
    >
      {#snippet trigger(props)}
        <IconButton
          {...props}
          {size}
          variant="ghost"
          color="secondary"
          icon={mdiDotsHorizontal}
          title=""
          aria-label="Preset actions"
        />
      {/snippet}
      <div class="flex flex-col">
        <MenuItem
          icon={mdiUpload}
          label={busy ? 'Importing…' : 'Import presets'}
          onclick={() => act(open)}
        />
        <MenuItem
          icon={mdiDownloadMultiple}
          label="Export all presets"
          onclick={() => act(presets.exportAll)}
        />
      </div>
    </Popover>
  {/snippet}
</FileImportButton>
