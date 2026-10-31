<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { scopes } from '#lib/stores/scopes.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import {
    developPanels,
    openDevelopPanels,
    SCOPES_PANEL,
    type PanelDef
  } from '#lib/panels/registry.ts';
  import { setDevelopPanel } from '#lib/panels/developPanels.ts';
  import { resetDevelopPanel } from '#lib/panels/panelReset.ts';
  import { modifiedDevelopPanels } from '#lib/editor/modified.ts';
  import Disclosure from '#lib/components/Disclosure.svelte';
  import ScopesSection from './scopes/ScopesSection.svelte';

  const openPanels = $derived(openDevelopPanels(ui.developOpenPanels));
  const modifiedPanels = $derived(modifiedDevelopPanels(editor.edits));

  function resetPanel(panel: PanelDef): void {
    editor.edits = resetDevelopPanel(editor.edits, panel.id);
    void editor.onCommit(`Reset ${panel.title}`);
  }
</script>

{#if !scopes.pinned && !ui.developModifiedOnly}
  <ScopesSection
    open={openPanels.has(SCOPES_PANEL)}
    onOpenChange={(open) => setDevelopPanel(SCOPES_PANEL, open)}
  />
{/if}
{#if ui.developModifiedOnly && modifiedPanels.size === 0}
  <div class="px-4 py-8 text-center text-xs text-dark/65">No modified adjustments</div>
{:else}
  {#each developPanels as panel (panel.id)}
    {#if !ui.developModifiedOnly || modifiedPanels.has(panel.id)}
      {@const Comp = panel.component}
      <Disclosure
        open={openPanels.has(panel.id)}
        title={panel.title}
        modified={modifiedPanels.has(panel.id)}
        onReset={() => resetPanel(panel)}
        onOpenChange={(open) => setDevelopPanel(panel.id, open)}
      >
        <div class="bg-black/10 px-3 pb-2 pt-1">
          <Comp />
        </div>
      </Disclosure>
    {/if}
  {/each}
{/if}
<div class="h-8"></div>
