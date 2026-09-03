<script lang="ts">
  import { Icon, Tooltip } from '@immich/ui';
  import {
    mdiBandage,
    mdiCropRotate,
    mdiExportVariant,
    mdiLayersTripleOutline,
    mdiTuneVariant
  } from '@mdi/js';
  import { modifiedTabCount } from '$lib/editorModified';
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui, type EditorTab } from '$lib/stores/ui.svelte';
  import { mergeProps } from '$lib/utils/mergeProps';

  const tools: { id: EditorTab; label: string; icon: string; title: string }[] = [
    {
      id: 'develop',
      label: 'Develop',
      icon: mdiTuneVariant,
      title: hint('Develop', 'openDevelop')
    },
    {
      id: 'masks',
      label: 'Masks',
      icon: mdiLayersTripleOutline,
      title: hint('Masks', 'openMasks')
    },
    { id: 'retouch', label: 'Retouch', icon: mdiBandage, title: hint('Retouch', 'openRetouch') },
    {
      id: 'geometry',
      label: 'Geometry',
      icon: mdiCropRotate,
      title: hint('Geometry', 'openGeometry')
    },
    { id: 'export', label: 'Export', icon: mdiExportVariant, title: hint('Export', 'openExport') }
  ];

  function onToolKeydown(event: KeyboardEvent, index: number): void {
    const offset = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (offset === 0) return;
    event.preventDefault();
    const next = tools[(index + offset + tools.length) % tools.length];
    if (!next) return;
    ui.openTab(next.id);
    document.getElementById(`editor-tool-${next.id}`)?.focus();
  }

  function toggleTool(tool: EditorTab): void {
    if (ui.editorTab === tool && !ui.rightCollapsed) {
      ui.togglePanels();
      return;
    }
    ui.openTab(tool);
  }
</script>

<div
  role="tablist"
  aria-orientation="vertical"
  aria-label="Editor tools"
  class="z-30 hidden w-12 shrink-0 flex-col items-center border-l border-hairline bg-editor-chrome md:flex"
>
  <div class="h-12 w-full shrink-0 border-b border-hairline"></div>
  <div class="flex w-full flex-1 flex-col items-center gap-1">
    {#each tools as tool, index (tool.id)}
      {@const current = ui.editorTab === tool.id}
      {@const active = ui.editorTab === tool.id && !ui.rightCollapsed}
      {@const modified = modifiedTabCount(editor.edits, tool.id) > 0}
      <Tooltip text={tool.title}>
        {#snippet child({ props })}
          <button
            type="button"
            role="tab"
            class="group relative flex h-11.25 w-10 items-center justify-center rounded-md transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-editor-chrome {tool.id ===
            'export'
              ? 'mt-auto'
              : ''} {active
              ? 'bg-primary/10 text-primary'
              : 'text-white/50 hover:bg-subtle hover:text-primary'}"
            aria-label={tool.label}
            aria-selected={active}
            aria-controls="editor-panel-{tool.id}"
            tabindex={current ? 0 : -1}
            {...mergeProps(props, {
              id: `editor-tool-${tool.id}`,
              onclick: () => toggleTool(tool.id),
              onkeydown: (event: KeyboardEvent) => onToolKeydown(event, index)
            })}
          >
            <Icon icon={tool.icon} size="21px" aria-hidden="true" />
            {#if modified}
              <span
                data-modified-indicator
                aria-hidden="true"
                class="absolute right-1 top-1 size-1.5 rounded-full bg-primary"
              ></span>
            {/if}
          </button>
        {/snippet}
      </Tooltip>
    {/each}
  </div>
</div>
