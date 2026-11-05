<script lang="ts">
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { help } from '#lib/commands/app.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import GlobalSearch from '#lib/components/GlobalSearch.svelte';
  import AttentionDot from '#lib/components/jobs/AttentionDot.svelte';
  import Wordmark from '#lib/components/Wordmark.svelte';
  import ThemeToggle from '#lib/components/shell/ThemeToggle.svelte';
  import {
    ControlBar,
    ControlBarContent,
    ControlBarHeader,
    ControlBarOverflow,
    IconButton
  } from '@immich/ui';
  import { mdiMenu, mdiCogOutline, mdiClose, mdiFormatListChecks } from '@mdi/js';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { SearchInput } from '#lib/browse/searchInput.svelte.ts';

  let { onToggleSidebar }: { onToggleSidebar?: () => void } = $props();

  const onSettings = $derived(page.url.pathname.startsWith('/settings'));

  const search = new SearchInput('/photos');

  function toggleSettings(): void {
    if (onSettings) {
      window.history.back();
    } else {
      void goto('/settings');
    }
  }
</script>

<ControlBar
  static
  variant="ghost"
  shape="rectangle"
  class="h-16! flex-nowrap select-none bg-light-50 px-2"
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
      onInput={search.inputEvent}
      onSubmit={search.submitEvent}
      onClear={search.clear}
      onFocus={search.focus}
    />
  </ControlBarContent>

  <ControlBarOverflow class="gap-0.5">
    <ThemeToggle />

    <CommandIconButton command={help} size="medium" variant="ghost" shape="rectangle" />

    <div class="relative">
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        shape="rectangle"
        icon={mdiFormatListChecks}
        title={jobs.attention ? 'Jobs: a job finished with failures' : 'Jobs'}
        aria-label="Jobs"
        onclick={jobs.toggle}
      />
      {#if jobs.activeCount > 0}
        <span
          class="pointer-events-none absolute -right-1 -top-1 min-w-4 rounded-full bg-primary px-1 text-center text-[10px] leading-4 text-light"
        >
          {jobs.activeCount}
        </span>
      {:else if jobs.attention}
        <AttentionDot class="absolute -right-0.5 -top-0.5" />
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
