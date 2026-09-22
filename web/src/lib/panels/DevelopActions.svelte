<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { openDevelopPanels } from '$lib/panels/registry';
  import { isNonGeometryIdentity } from '$lib/types/edits';
  import { modifiedDevelopPanels } from '$lib/editorModified';
  import HistoryPopover from '$lib/components/editor/HistoryPopover.svelte';
  import { hint } from '$lib/keybinds';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiAutoFix,
    mdiRestore,
    mdiFilterVariant,
    mdiContentCopy,
    mdiContentPaste
  } from '@mdi/js';

  const neutral = $derived(isNonGeometryIdentity(editor.edits));

  function toggleModifiedOnly(): void {
    ui.developModifiedOnly = !ui.developModifiedOnly;
    if (!ui.developModifiedOnly) return;
    const open = openDevelopPanels(ui.developOpenPanels);
    ui.setDevelopPanels([...new Set([...open, ...modifiedDevelopPanels(editor.edits)])]);
  }
</script>

<div class="flex min-w-0 flex-1 items-center gap-0.5">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-white/6"
    leadingIcon={mdiAutoFix}
    title={hint('Auto adjust tone', 'autoAdjust')}
    aria-label="Auto"
    disabled={editor.autoBusy || !editor.assetId}
    onclick={() => void editor.onAutoAdjust()}
  >
    {editor.autoBusy ? 'Analyzing…' : 'Auto'}
  </Button>
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-white/6"
    leadingIcon={mdiRestore}
    title={hint('Reset edits', 'resetEdits')}
    aria-label="Reset edits"
    disabled={neutral || editor.saving}
    onclick={() => void editor.onReset()}
  >
    Reset
  </Button>
</div>
<div class="h-5 w-px shrink-0 bg-hairline"></div>
<div class="flex shrink-0 items-center gap-0.5">
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class="bg-transparent hover:bg-white/6"
    icon={mdiContentCopy}
    title={editor.hasEdits ? hint('Copy edits', 'copyEdits') : 'Nothing to copy'}
    disabled={!editor.assetId || !editor.hasEdits}
    aria-label="Copy edits"
    onclick={editor.copyEdits}
  />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class="bg-transparent hover:bg-white/6"
    icon={mdiContentPaste}
    title={editor.hasClipboard ? hint('Paste edits', 'pasteEdits') : 'Nothing copied'}
    disabled={!editor.assetId || !editor.hasClipboard || editor.saving}
    aria-label="Paste edits"
    onclick={() => void editor.pasteEdits()}
  />
  <HistoryPopover />
  <IconButton
    size="small"
    variant="ghost"
    color={ui.developModifiedOnly ? 'primary' : 'secondary'}
    class="bg-transparent hover:bg-white/6"
    icon={mdiFilterVariant}
    title={ui.developModifiedOnly ? 'Show all adjustments' : 'Show modified only'}
    aria-label={ui.developModifiedOnly ? 'Show all adjustments' : 'Show modified only'}
    aria-pressed={ui.developModifiedOnly}
    onclick={toggleModifiedOnly}
  />
</div>
