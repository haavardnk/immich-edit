<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { panels } from '$lib/panels/registry';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let openPanels = $state(new Set(panels.filter((p) => p.defaultOpen).map((p) => p.id)));

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
  class:w-0={ui.rightCollapsed}
>
  {#if !ui.rightCollapsed}
    {#if editor.assetId}
      <div class="px-4 py-2.5 text-[10px] uppercase tracking-widest text-immich-dark-fg/40 font-semibold">
        Develop
      </div>
      <div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden">
        {#each panels as panel (panel.id)}
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
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center text-xs text-immich-dark-fg/30 px-4 text-center">
        Select an asset to edit
      </div>
    {/if}
  {/if}
</aside>
