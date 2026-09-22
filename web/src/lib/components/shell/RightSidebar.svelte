<script lang="ts">
  import {
    ui,
    MAX_INSPECTOR_WIDTH,
    MIN_INSPECTOR_WIDTH,
    type EditorTab
  } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { scopes } from '$lib/stores/scopes.svelte';
  import { openDevelopPanels, SCOPES_PANEL } from '$lib/panels/registry';
  import { setDevelopPanel } from '$lib/panels/developPanels';
  import DevelopActions from '$lib/panels/DevelopActions.svelte';
  import DevelopPanels from '$lib/panels/DevelopPanels.svelte';
  import ScopesSection from '$lib/panels/scopes/ScopesSection.svelte';
  import TransformPanel from '$lib/panels/Transform.svelte';
  import ExportPanel from '$lib/panels/Export.svelte';
  import MasksPanel from '$lib/panels/Masks.svelte';
  import RetouchPanel from '$lib/panels/Retouch.svelte';
  import ResizeHandle from './ResizeHandle.svelte';

  const editorTabs: { id: EditorTab; label: string }[] = [
    { id: 'develop', label: 'Develop' },
    { id: 'masks', label: 'Masks' },
    { id: 'retouch', label: 'Retouch' },
    { id: 'geometry', label: 'Geometry' },
    { id: 'export', label: 'Export' }
  ];

  const scopesOpen = $derived(openDevelopPanels(ui.developOpenPanels).has(SCOPES_PANEL));
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

  $effect(() => {
    scopes.setPanelOpen(scopesOpen);
  });
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
            {activeEditorTab?.label}
          </h2>
        </div>
      </div>

      {#if ui.editorTab === 'develop'}
        <div class="flex shrink-0 items-center gap-1.5 border-b border-hairline px-3 py-1.5">
          <DevelopActions />
        </div>
        {#if scopes.pinned}
          <div class="shrink-0">
            <ScopesSection
              open={scopesOpen}
              onOpenChange={(open) => setDevelopPanel(SCOPES_PANEL, open)}
            />
          </div>
        {/if}
      {/if}

      <div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
        {#if ui.editorTab === 'develop'}
          <DevelopPanels />
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
