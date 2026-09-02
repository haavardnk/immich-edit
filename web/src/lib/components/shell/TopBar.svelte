<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { hint } from '$lib/keybinds';
  import GlobalSearch from '$lib/components/GlobalSearch.svelte';
  import Wordmark from '$lib/components/Wordmark.svelte';
  import {
    ControlBar,
    ControlBarContent,
    ControlBarHeader,
    ControlBarOverflow,
    IconButton
  } from '@immich/ui';
  import {
    mdiMenu,
    mdiCogOutline,
    mdiClose,
    mdiFormatListChecks,
    mdiKeyboardOutline
  } from '@mdi/js';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { untrack } from 'svelte';
  import { jobs } from '$lib/stores/jobs.svelte';

  let { onToggleSidebar }: { onToggleSidebar?: () => void } = $props();

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

  function onSearchSubmit(event: SubmitEvent): void {
    event.preventDefault();
    clearTimeout(searchTimeout);
    submitSearch(ui.searchQuery);
  }

  function clearSearch(): void {
    clearTimeout(searchTimeout);
    ui.searchQuery = '';
    if (page.url.pathname === '/search') goto('/photos');
  }

  function toggleSettings(): void {
    if (onSettings) {
      window.history.back();
    } else {
      goto('/settings');
    }
  }
</script>

<ControlBar
  static
  variant="ghost"
  shape="rectangle"
  class="h-16! select-none bg-neutral-950 px-2"
  aria-label="Global navigation"
>
  <ControlBarHeader class="flex-row items-center gap-2 lg:w-60">
    {#if onToggleSidebar}
      <IconButton
        id="library-menu"
        size="medium"
        variant="ghost"
        color="secondary"
        shape="round"
        class="lg:hidden"
        icon={mdiMenu}
        title={onSettings ? 'Administration' : 'Library'}
        aria-label={onSettings ? 'Administration' : 'Library'}
        onclick={onToggleSidebar}
      />
    {/if}
    <a href="/" class="hidden hover:opacity-80 lg:flex">
      <Wordmark bright />
    </a>
  </ControlBarHeader>

  <ControlBarContent class="min-w-0 justify-start ps-1 pe-3">
    <GlobalSearch
      value={ui.searchQuery}
      onInput={onSearchInput}
      onSubmit={onSearchSubmit}
      onClear={clearSearch}
    />
  </ControlBarContent>

  <ControlBarOverflow class="gap-0.5">
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="rectangle"
      icon={mdiKeyboardOutline}
      title={hint('Keyboard shortcuts', 'help')}
      aria-label="Keyboard shortcuts"
      onclick={() => ui.toggleKeybindsHelp()}
    />

    <div class="relative">
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        shape="rectangle"
        icon={mdiFormatListChecks}
        title="Jobs"
        aria-label="Jobs"
        onclick={jobs.toggle}
      />
      {#if jobs.activeCount > 0}
        <span
          class="pointer-events-none absolute -right-1 -top-1 min-w-4 rounded-full bg-primary px-1 text-center text-[10px] leading-4 text-light"
        >
          {jobs.activeCount}
        </span>
      {/if}
    </div>

    <IconButton
      size="medium"
      variant="ghost"
      color={onSettings ? 'primary' : 'secondary'}
      shape="rectangle"
      icon={onSettings ? mdiClose : mdiCogOutline}
      title={onSettings ? 'Close settings' : 'Settings & diagnostics'}
      aria-label={onSettings ? 'Close settings' : 'Settings'}
      aria-pressed={onSettings}
      onclick={toggleSettings}
    />
  </ControlBarOverflow>
</ControlBar>
