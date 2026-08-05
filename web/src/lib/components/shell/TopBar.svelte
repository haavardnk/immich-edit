<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { album } from '$lib/stores/album.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { hint } from '$lib/keybinds';
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { mdiLoading, mdiCogOutline, mdiClose, mdiFormatListChecks, mdiKeyboardOutline, mdiMagnify } from '@mdi/js';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { untrack } from 'svelte';
  import { jobs } from '$lib/stores/jobs.svelte';

  const subtitle = $derived(
    !editor.assetId && album.current ? album.current.albumName : ''
  );

  const onSettings = $derived(page.url.pathname.startsWith('/settings'));

  const routeQuery = $derived(
    page.url.pathname === '/search' ? (page.url.searchParams.get('q') ?? '') : null
  );

  let searchTimeout: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const q = routeQuery;
    if (q !== null) untrack(() => (ui.searchQuery = q));
  });

  function submitSearch(value: string): void {
    const q = value.trim();
    if (q) {
      goto('/search?q=' + encodeURIComponent(q), {
        replaceState: page.url.pathname === '/search',
        keepFocus: true
      });
    } else if (page.url.pathname === '/search') {
      goto('/photos', { keepFocus: true });
    }
  }

  function onSearchInput(e: Event): void {
    const value = (e.currentTarget as HTMLInputElement).value;
    ui.searchQuery = value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => submitSearch(value), 300);
  }

  function onSearchKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      clearTimeout(searchTimeout);
      submitSearch((e.currentTarget as HTMLInputElement).value);
    }
  }

  function clearSearch(): void {
    clearTimeout(searchTimeout);
    ui.searchQuery = '';
    if (page.url.pathname === '/search') goto('/photos');
  }

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

  {#if editor.assetId}
    <div class="flex-1"></div>
  {:else}
    <div class="flex-1 flex justify-center px-2">
      <div class="relative w-full max-w-md">
        <div class="absolute left-2 top-1/2 -translate-y-1/2 text-immich-dark-fg/40">
          <Icon path={mdiMagnify} size={16} />
        </div>
        <input
          type="text"
          placeholder="Search your photos"
          value={ui.searchQuery}
          oninput={onSearchInput}
          onkeydown={onSearchKeydown}
          class="w-full h-7 bg-white/5 rounded-full pl-8 pr-7 text-xs text-immich-dark-fg placeholder:text-immich-dark-fg/40 outline-none focus:bg-white/10 transition-colors"
        />
        {#if ui.searchQuery}
          <button
            type="button"
            onclick={clearSearch}
            class="absolute right-1.5 top-1/2 -translate-y-1/2 p-0.5 rounded-full hover:bg-white/10 text-immich-dark-fg/50 hover:text-immich-dark-fg"
            title="Clear search"
            aria-label="Clear search"
          >
            <Icon path={mdiClose} size={14} />
          </button>
        {/if}
      </div>
    </div>
  {/if}

  {#if editor.pending}
    <Icon path={mdiLoading} size={16} class="animate-spin text-immich-dark-primary/70" />
  {/if}

  <button
    type="button"
    onclick={() => ui.toggleKeybindsHelp()}
    class="p-1.5 hover:bg-white/10 rounded transition-colors text-immich-dark-fg/60 hover:text-immich-dark-fg"
    title={hint('Keyboard shortcuts', 'help')}
    aria-label="Keyboard shortcuts"
  >
    <Icon path={mdiKeyboardOutline} size={16} />
  </button>

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


