<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiViewDashboardOutline,
    mdiTuneVertical,
    mdiFilmstrip,
    mdiLoading,
  } from '@mdi/js';

  const subtitle = $derived(
    editor.assetId
      ? (editor.asset?.originalFileName ?? editor.assetId)
      : album.current
        ? album.current.albumName
        : ''
  );
</script>

<header
  class="flex items-center h-11 bg-immich-dark-gray border-b border-white/10 px-3 gap-2 flex-none select-none"
>
  <button
    class="p-1 rounded-md hover:bg-white/10 transition-colors"
    onclick={ui.toggleLeft}
    aria-label="toggle library panel"
    title="Library"
  >
    <Icon path={mdiViewDashboardOutline} size={18} class="opacity-80" />
  </button>

  <a href="/" class="text-sm font-semibold tracking-tight text-immich-dark-primary hover:opacity-80 transition-opacity">
    immich-edit
  </a>

  <div class="flex-1 text-xs text-immich-dark-fg/50 truncate px-2">
    {subtitle}
  </div>

  {#if editor.pending}
    <Icon path={mdiLoading} size={16} class="animate-spin text-immich-dark-primary/70" />
  {/if}

  {#if editor.assetId}
    <button
      class="p-1 rounded-md hover:bg-white/10 transition-colors {!ui.filmstripCollapsed ? 'bg-white/10' : ''}"
      onclick={ui.toggleFilmstrip}
      aria-label="toggle filmstrip"
      title="Filmstrip"
    >
      <Icon path={mdiFilmstrip} size={18} class="opacity-80" />
    </button>
  {/if}

  <button
    class="p-1 rounded-md hover:bg-white/10 transition-colors"
    onclick={ui.toggleRight}
    aria-label="toggle edit panel"
    title="Develop"
  >
    <Icon path={mdiTuneVertical} size={18} class="opacity-80" />
  </button>
</header>
