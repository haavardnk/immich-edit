<script lang="ts">
  import {
    ui,
    MAX_INSPECTOR_WIDTH,
    MIN_INSPECTOR_WIDTH,
    type EditorTab
  } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { developPanels } from '$lib/panels/registry';
  import { isNonGeometryIdentity } from '$lib/types/edits';
  import { modifiedDevelopPanels } from '$lib/editorModified';
  import TransformPanel from '$lib/panels/Transform.svelte';
  import ExportPanel from '$lib/panels/Export.svelte';
  import MasksPanel from '$lib/panels/Masks.svelte';
  import RetouchPanel from '$lib/panels/Retouch.svelte';
  import Disclosure from '$lib/components/Disclosure.svelte';
  import HistoryPopover from '$lib/components/editor/HistoryPopover.svelte';
  import ResizeHandle from './ResizeHandle.svelte';
  import { hint } from '$lib/keybinds';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiAutoFix,
    mdiRestore,
    mdiFilterVariant,
    mdiContentCopy,
    mdiContentPaste
  } from '@mdi/js';

  const editorTabs: { id: EditorTab; label: string }[] = [
    { id: 'develop', label: 'Develop' },
    { id: 'masks', label: 'Masks' },
    { id: 'retouch', label: 'Retouch' },
    { id: 'geometry', label: 'Geometry' },
    { id: 'export', label: 'Export' }
  ];

  let openPanels = $state(new Set(developPanels.filter((p) => p.defaultOpen).map((p) => p.id)));
  let modifiedOnly = $state(false);
  const neutral = $derived(isNonGeometryIdentity(editor.edits));
  const modifiedPanels = $derived(modifiedDevelopPanels(editor.edits));
  const activeEditorTab = $derived(
    editorTabs.find((tab) => tab.id === ui.editorTab) ?? editorTabs[0]
  );

  $effect(() => {
    if (ui.editorTab !== 'masks') {
      editor.setActiveLayer(null);
      editor.setActiveMaskComponent(null);
    }
    if (ui.editorTab !== 'retouch') editor.activeRetouchId = null;
  });

  function setPanel(id: string, open: boolean): void {
    if (open) {
      openPanels.add(id);
    } else {
      openPanels.delete(id);
    }
    openPanels = new Set(openPanels);
  }

  function toggleModifiedOnly(): void {
    modifiedOnly = !modifiedOnly;
    if (!modifiedOnly) return;
    openPanels = new Set([...openPanels, ...modifiedPanels]);
  }
</script>

<aside
  aria-label="Editor controls"
  class="relative hidden min-h-0 shrink-0 flex-col border-l border-hairline bg-editor-panel md:flex"
  style:width={`${ui.inspectorWidth}px`}
  style:display={ui.rightCollapsed ? 'none' : undefined}
>
  <ResizeHandle
    label="Resize editor controls"
    orientation="horizontal"
    value={ui.inspectorWidth}
    min={MIN_INSPECTOR_WIDTH}
    max={MAX_INSPECTOR_WIDTH}
    step={16}
    shiftStep={32}
    class="after:bg-hairline absolute inset-y-0 left-0 z-40 w-3 -translate-x-1/2 cursor-col-resize bg-transparent outline-none after:absolute after:inset-y-0 after:left-1/2 after:w-px after:-translate-x-1/2 after:transition-colors hover:after:bg-primary focus-visible:after:bg-primary"
    activeClass="after:bg-primary"
    onLive={ui.setInspectorWidth}
    onCommit={ui.persistEditorUi}
  />
  {#if editor.assetId}
    <div
      id="editor-panel-{ui.editorTab}"
      role="tabpanel"
      aria-labelledby="editor-tool-{ui.editorTab}"
      class="flex min-h-0 flex-1 flex-col"
    >
      <div class="flex h-12 shrink-0 items-center gap-2 border-b border-hairline px-3">
        <div class="min-w-0 flex-1">
          <h2 class="truncate text-[13px] font-semibold text-white/90">
            {activeEditorTab.label}
          </h2>
        </div>
      </div>

      {#if ui.editorTab === 'develop'}
        <div class="flex shrink-0 items-center gap-1.5 border-b border-hairline px-3 py-1.5">
          <div class="flex min-w-0 flex-1 items-center gap-0.5">
            <Button
              size="tiny"
              variant="ghost"
              color="secondary"
              class="h-8 min-w-0 flex-1 justify-center bg-transparent hover:bg-white/6"
              leadingIcon={mdiAutoFix}
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
              color={modifiedOnly ? 'primary' : 'secondary'}
              class="bg-transparent hover:bg-white/6"
              icon={mdiFilterVariant}
              title={modifiedOnly ? 'Show all adjustments' : 'Show modified only'}
              aria-label={modifiedOnly ? 'Show all adjustments' : 'Show modified only'}
              aria-pressed={modifiedOnly}
              onclick={toggleModifiedOnly}
            />
          </div>
        </div>
      {/if}

      <div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
        {#if ui.editorTab === 'develop'}
          {#if modifiedOnly && modifiedPanels.size === 0}
            <div class="px-4 py-8 text-center text-xs text-dark/65">No modified adjustments</div>
          {:else}
            {#each developPanels as panel (panel.id)}
              {#if !modifiedOnly || modifiedPanels.has(panel.id)}
                {@const Comp = panel.component}
                <Disclosure
                  open={openPanels.has(panel.id)}
                  title={panel.title}
                  modified={modifiedPanels.has(panel.id)}
                  onOpenChange={(v) => setPanel(panel.id, v)}
                >
                  <div class="bg-black/10 {panel.id === 'histogram' ? '' : 'px-3 pb-2 pt-1'}">
                    <Comp />
                  </div>
                </Disclosure>
              {/if}
            {/each}
          {/if}
          <div class="h-8"></div>
        {:else if ui.editorTab === 'masks'}
          <div class="px-3 py-2">
            <MasksPanel />
          </div>
        {:else if ui.editorTab === 'retouch'}
          <div class="px-3 py-2">
            <RetouchPanel />
          </div>
        {:else if ui.editorTab === 'geometry'}
          <div class="px-3 py-2">
            <TransformPanel />
          </div>
        {:else if ui.editorTab === 'export'}
          <div class="px-3 py-2">
            <ExportPanel />
          </div>
        {/if}
      </div>
    </div>
  {/if}
</aside>
