<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { panels } from '$lib/panels/registry';
</script>

<aside
  class="bg-base-300 border-l border-base-content/10 flex flex-col min-h-0 transition-[width] duration-150 ease-out"
  class:w-72={!ui.rightCollapsed}
  class:w-0={ui.rightCollapsed}
>
  {#if !ui.rightCollapsed}
    {#if editor.assetId}
      <div class="px-3 py-2 text-[10px] uppercase tracking-widest opacity-50 font-semibold">
        Develop
      </div>
      <div class="flex-1 min-h-0 overflow-y-auto no-scrollbar">
        {#each panels as panel (panel.id)}
          {@const Comp = panel.component}
          <details class="border-t border-base-content/10" open={panel.defaultOpen}>
            <summary
              class="px-3 py-1.5 text-[11px] uppercase tracking-wider opacity-70 cursor-pointer hover:bg-base-content/5 select-none"
            >
              {panel.title}
            </summary>
            <div class="px-3 pb-3 pt-1">
              <Comp />
            </div>
          </details>
        {/each}
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center text-xs opacity-40 px-4 text-center">
        Select an asset to edit
      </div>
    {/if}
  {/if}
</aside>
