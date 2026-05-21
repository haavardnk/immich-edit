<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import AlbumList from '$lib/components/library/AlbumList.svelte';
</script>

<aside
  class="bg-base-300 border-r border-base-content/10 flex flex-col min-h-0 transition-[width] duration-150 ease-out"
  class:w-64={!ui.leftCollapsed}
  class:w-0={ui.leftCollapsed}
>
  {#if !ui.leftCollapsed}
    <div class="px-3 py-2 text-[10px] uppercase tracking-widest opacity-50 font-semibold">
      Library
    </div>
    <div class="flex-1 min-h-0 overflow-y-auto no-scrollbar">
      <AlbumList />
    </div>
    <div class="px-3 py-2 text-[10px] opacity-40 border-t border-base-content/10">
      Albums
    </div>
  {/if}
</aside>
