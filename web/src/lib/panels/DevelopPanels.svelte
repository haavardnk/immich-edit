<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { scopes } from '#lib/stores/scopes.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { developPanels, openDevelopPanels, SCOPES_PANEL } from '#lib/panels/registry.ts';
  import { setDevelopPanel } from '#lib/panels/developPanels.ts';
  import { commitPanelReset } from '#lib/components/editor/controls/paramReset.ts';
  import { modifiedDevelopPanels } from '#lib/editor/modified.ts';
  import Disclosure from '#lib/components/Disclosure.svelte';
  import { provideSettingFilter } from '#lib/components/editor/controls/settingFilter.ts';
  import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
  import ScopesSection from './scopes/ScopesSection.svelte';
  import TargetButton from '#lib/components/editor/controls/TargetButton.svelte';
  import { pinnedParams } from '#lib/editor/quickSettings.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import QuickPanel from './quick/QuickPanel.svelte';

  provideSettingFilter(settingSearch);

  const pinned = $derived(pinnedParams(preferences.pins));
  const browsing = $derived(!ui.developModifiedOnly && !settingSearch.filtering);

  const openPanels = $derived(openDevelopPanels(ui.developOpenPanels));
  const modifiedPanels = $derived(modifiedDevelopPanels(editor.edits));
  const matchedPanels = $derived(
    new Set(settingSearch.matches.map((match) => match.setting.panel))
  );
  const shownPanels = $derived(
    developPanels.filter(
      (panel) =>
        (!ui.developModifiedOnly || modifiedPanels.has(panel.id)) &&
        (!settingSearch.filtering || matchedPanels.has(panel.id))
    )
  );
</script>

{#if !scopes.pinned && browsing}
  <ScopesSection
    open={openPanels.has(SCOPES_PANEL)}
    onOpenChange={(open) => setDevelopPanel(SCOPES_PANEL, open)}
  />
{/if}
{#if browsing && pinned.length > 0}
  <QuickPanel params={pinned} />
{/if}
{#if shownPanels.length === 0}
  <div class="px-4 py-8 text-center text-xs text-dark/65">
    {settingSearch.filtering ? 'No matching settings' : 'No modified adjustments'}
  </div>
{:else}
  {#each shownPanels as panel (panel.id)}
    {@const Comp = panel.component}
    {@const Actions = panel.actions}
    {@const target = panel.target}
    <div data-panel={panel.id}>
      {#snippet panelActions()}
        {#if target}
          <TargetButton tool={target} />
        {/if}
        {#if Actions}
          <Actions />
        {/if}
      {/snippet}
      <Disclosure
        open={settingSearch.filtering || openPanels.has(panel.id)}
        title={panel.title}
        modified={modifiedPanels.has(panel.id)}
        onReset={() => commitPanelReset(panel)}
        actions={Actions || target ? panelActions : undefined}
        onOpenChange={(open) => setDevelopPanel(panel.id, open)}
      >
        <div class="bg-recess px-3 pb-2 pt-1">
          <Comp />
        </div>
      </Disclosure>
    </div>
  {/each}
{/if}
<div class="h-8"></div>
