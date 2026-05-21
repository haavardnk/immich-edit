<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { developPanels } from '$lib/panels/registry';
  import { isIdentity } from '$lib/types/edits';
  import TransformPanel from '$lib/panels/Transform.svelte';
  import ExportPanel from '$lib/panels/Export.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiChevronDown,
    mdiChevronRight,
    mdiChevronLeft,
    mdiAutoFix,
    mdiRestore,
  } from '@mdi/js';

  type Tab = 'develop' | 'geometry' | 'export';
  let activeTab = $state<Tab>('develop');
  let openPanels = $state(new Set(developPanels.filter((p) => p.defaultOpen).map((p) => p.id)));
  const neutral = $derived(isIdentity(editor.edits));

  function toggle(id: string): void {
    if (openPanels.has(id)) {
      openPanels.delete(id);
    } else {
      openPanels.add(id);
    }
    openPanels = new Set(openPanels);
  }
</script>

<aside
  class="bg-immich-dark-gray border-l border-white/5 flex flex-col min-h-0 transition-[width] duration-200 ease-out"
  class:w-72={!ui.rightCollapsed}
  class:w-7={ui.rightCollapsed}
>
  {#if ui.rightCollapsed}
    <button
      class="flex-1 flex items-center justify-center hover:bg-white/5 transition-colors"
      onclick={ui.toggleRight}
      aria-label="expand edit panel"
      title="Develop"
    >
      <Icon path={mdiChevronLeft} size={16} class="opacity-40" />
    </button>
  {:else}
    {#if editor.assetId}
      <div class="flex items-center border-b border-white/10">
        <nav class="flex flex-1">
          {#each [{ id: 'develop', label: 'Develop' }, { id: 'geometry', label: 'Geometry' }, { id: 'export', label: 'Export' }] as tab (tab.id)}
            <button
              class="flex-1 py-2 text-[11px] uppercase tracking-wider transition-colors {activeTab === tab.id ? 'text-immich-dark-primary border-b-2 border-immich-dark-primary' : 'text-immich-dark-fg/40 hover:text-immich-dark-fg/60'}"
              onclick={() => (activeTab = tab.id as Tab)}
            >
              {tab.label}
            </button>
          {/each}
        </nav>
        <button
          class="p-1.5 hover:bg-white/10 transition-colors"
          onclick={ui.toggleRight}
          aria-label="collapse edit panel"
          title="Collapse"
        >
          <Icon path={mdiChevronRight} size={14} class="opacity-40" />
        </button>
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
        {#if activeTab === 'develop'}
          {#each developPanels as panel (panel.id)}
            {@const Comp = panel.component}
            {@const isOpen = openPanels.has(panel.id)}
            <div class="border-t border-white/5">
              <button
                class="w-full flex items-center gap-1.5 px-4 py-2 text-[11px] uppercase tracking-wider text-immich-dark-fg/60 hover:bg-white/5 transition-colors select-none"
                onclick={() => toggle(panel.id)}
              >
                <Icon path={isOpen ? mdiChevronDown : mdiChevronRight} size={14} class="opacity-50" />
                {panel.title}
              </button>
              {#if isOpen}
                <div class="px-4 pb-3 pt-1">
                  <Comp />
                </div>
              {/if}
            </div>
          {/each}

          <div class="border-t border-white/5 px-4 py-3 flex flex-col gap-2">
            <button
              class="flex items-center justify-center gap-2 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              disabled={editor.autoBusy || !editor.assetId}
              onclick={() => void editor.onAutoAdjust()}
            >
              <Icon path={mdiAutoFix} size={16} />
              {editor.autoBusy ? 'Analyzing…' : 'Auto adjust'}
            </button>
            <button
              class="flex items-center justify-center gap-2 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
              disabled={neutral || editor.saving}
              onclick={() => void editor.onReset()}
            >
              <Icon path={mdiRestore} size={16} />
              Reset all
            </button>
            <div class="text-[10px] text-immich-dark-fg/30 text-center">
              {#if editor.saving}saving…{:else if neutral}no edits{:else}edited{/if}
            </div>
          </div>
        {:else if activeTab === 'geometry'}
          <div class="px-4 py-3">
            <TransformPanel />
          </div>
        {:else if activeTab === 'export'}
          <ExportPanel />
        {/if}
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center text-xs text-immich-dark-fg/30 px-4 text-center">
        Select an asset to edit
      </div>
    {/if}
  {/if}
</aside>
