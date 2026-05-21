<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';

  const subtitle = $derived(
    editor.assetId
      ? (editor.asset?.originalFileName ?? editor.assetId)
      : album.current
        ? album.current.albumName
        : ''
  );
</script>

<header
  class="navbar min-h-10 h-10 bg-base-300 border-b border-base-content/10 px-2 gap-1 flex-none"
>
  <button
    class="btn btn-ghost btn-xs btn-square"
    onclick={ui.toggleLeft}
    aria-label="toggle library panel"
    title="Library panel"
  >
    {#if ui.leftCollapsed}❯{:else}❮{/if}
  </button>
  <a href="/" class="btn btn-ghost btn-xs normal-case font-semibold tracking-tight">
    immich-edit
  </a>
  <div class="flex-1 text-xs opacity-60 truncate px-2">
    {subtitle}
  </div>
  {#if editor.pending}
    <span class="loading loading-spinner loading-xs text-warning" title="rendering"></span>
  {/if}
  <button
    class="btn btn-ghost btn-xs btn-square"
    onclick={ui.toggleRight}
    aria-label="toggle edit panel"
    title="Edit panel"
  >
    {#if ui.rightCollapsed}❮{:else}❯{/if}
  </button>
</header>
