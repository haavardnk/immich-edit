<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { openDevelopPanels } from '#lib/panels/registry.ts';
  import { modifiedDevelopPanels } from '#lib/editor/modified.ts';
  import HistoryPopover from '#lib/components/editor/HistoryPopover.svelte';
  import { commandLabel, commandTitle, isEnabled, runCommand } from '#lib/commands/command.ts';
  import { autoAdjust, copyEdits, pasteEdits, resetDevelop } from '#lib/commands/editor/edits.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiFilterVariant } from '@mdi/js';

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
    class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-light-950/6"
    leadingIcon={autoAdjust.icon}
    title={commandTitle(autoAdjust)}
    aria-label="Auto"
    disabled={!isEnabled(autoAdjust)}
    onclick={() => runCommand(autoAdjust)}
  >
    {editor.autoRunning === 'all' ? 'Analyzing…' : 'Auto'}
  </Button>
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-light-950/6"
    leadingIcon={resetDevelop.icon}
    title={commandTitle(resetDevelop)}
    aria-label={commandLabel(resetDevelop)}
    disabled={!isEnabled(resetDevelop)}
    onclick={() => runCommand(resetDevelop)}
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
    class="bg-transparent hover:bg-light-950/6"
    icon={copyEdits.icon}
    title={commandTitle(copyEdits)}
    disabled={!isEnabled(copyEdits)}
    aria-label={commandLabel(copyEdits)}
    onclick={() => runCommand(copyEdits)}
  />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class="bg-transparent hover:bg-light-950/6"
    icon={pasteEdits.icon}
    title={commandTitle(pasteEdits)}
    disabled={!isEnabled(pasteEdits)}
    aria-label={commandLabel(pasteEdits)}
    onclick={() => runCommand(pasteEdits)}
  />
  <HistoryPopover />
  <IconButton
    size="small"
    variant="ghost"
    color={ui.developModifiedOnly ? 'primary' : 'secondary'}
    class="bg-transparent hover:bg-light-950/6"
    icon={mdiFilterVariant}
    title={ui.developModifiedOnly ? 'Show all adjustments' : 'Show modified only'}
    aria-label={ui.developModifiedOnly ? 'Show all adjustments' : 'Show modified only'}
    aria-pressed={ui.developModifiedOnly}
    onclick={toggleModifiedOnly}
  />
</div>
