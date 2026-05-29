<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiLoading, mdiCogOutline, mdiClose } from '@mdi/js';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';

  const subtitle = $derived(
    editor.assetId
      ? (editor.asset?.originalFileName ?? editor.assetId)
      : album.current
        ? album.current.albumName
        : ''
  );

  const onSettings = $derived(page.url.pathname.startsWith('/settings'));

  function toggleSettings(): void {
    if (onSettings) {
      if (window.history.length > 1) window.history.back();
      else goto('/');
    } else {
      goto('/settings');
    }
  }
</script>

<header
  class="flex items-center h-11 bg-immich-dark-gray border-b border-white/10 px-3 gap-2 flex-none select-none"
>
  <a href="/" class="text-sm font-semibold tracking-tight text-immich-dark-primary hover:opacity-80 transition-opacity">
    immich-edit
  </a>

  <div class="flex-1 text-xs text-immich-dark-fg/50 truncate px-2">
    {subtitle}
  </div>

  {#if editor.pending}
    <Icon path={mdiLoading} size={16} class="animate-spin text-immich-dark-primary/70" />
  {/if}

  <button
    type="button"
    onclick={toggleSettings}
    class="p-1.5 hover:bg-white/10 rounded transition-colors {onSettings ? 'text-immich-dark-fg' : 'text-immich-dark-fg/60 hover:text-immich-dark-fg'}"
    title={onSettings ? 'Close settings' : 'Settings & diagnostics'}
    aria-label={onSettings ? 'Close settings' : 'Settings'}
    aria-pressed={onSettings}
  >
    <Icon path={onSettings ? mdiClose : mdiCogOutline} size={16} />
  </button>
</header>


