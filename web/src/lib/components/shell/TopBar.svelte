<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { mdiLoading, mdiCogOutline, mdiClose, mdiFormatListChecks } from '@mdi/js';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { jobs } from '$lib/stores/jobs.svelte';

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
  <a href="/" class="flex items-center gap-2 text-sm font-semibold tracking-tight hover:opacity-80 transition-opacity">
    <Logo size={18} />
    <span><span class="text-immich-dark-fg/90">immich</span><span style="color:#6366F1">-edit</span></span>
  </a>

  <div class="flex-1 text-xs text-immich-dark-fg/50 truncate px-2">
    {subtitle}
  </div>

  {#if editor.pending}
    <Icon path={mdiLoading} size={16} class="animate-spin text-immich-dark-primary/70" />
  {/if}

  <button
    type="button"
    onclick={jobs.toggle}
    class="relative p-1.5 hover:bg-white/10 rounded transition-colors text-immich-dark-fg/60 hover:text-immich-dark-fg"
    title="Jobs"
    aria-label="Jobs"
  >
    <Icon path={mdiFormatListChecks} size={16} />
    {#if jobs.activeCount > 0}
      <span
        class="absolute -top-0.5 -right-0.5 min-w-3.5 h-3.5 px-1 rounded-full bg-immich-primary text-[9px] leading-3.5 text-white text-center"
      >
        {jobs.activeCount}
      </span>
    {/if}
  </button>

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


