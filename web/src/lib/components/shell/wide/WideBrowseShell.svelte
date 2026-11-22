<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, type Snippet } from 'svelte';
  import GlobalSearch from '#lib/components/GlobalSearch.svelte';
  import Wordmark from '#lib/components/Wordmark.svelte';
  import LeftSidebar from '#lib/components/shell/LeftSidebar.svelte';
  import SettingsSidebar from '#lib/components/shell/SettingsSidebar.svelte';
  import AccountDrawer from '#lib/components/shell/account/AccountDrawer.svelte';
  import { accountDrawer } from '#lib/components/shell/account/accountDrawer.svelte.ts';
  import SidebarFoot from '#lib/components/shell/wide/SidebarFoot.svelte';
  import WideSidebarToggle from '#lib/components/shell/wide/WideSidebarToggle.svelte';
  import { wideSidebar } from '#lib/components/shell/wide/wideSidebar.svelte.ts';
  import JobsDock from '#lib/components/jobs/JobsDock.svelte';
  import { SearchInput } from '#lib/browse/searchInput.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let { children }: { children: Snippet } = $props();

  const settingsRoute = $derived(page.url.pathname.startsWith('/settings'));

  const search = new SearchInput('/photos');

  onDestroy(() => accountDrawer.close());
</script>

<div class="app-frame flex overflow-hidden">
  <div
    class="h-full shrink-0 overflow-hidden motion-safe:transition-[width] motion-safe:duration-base motion-safe:ease-out {wideSidebar.open
      ? 'w-70'
      : 'w-0'}"
  >
    <div
      class="flex h-full w-70 flex-col border-e border-hairline bg-light-50 select-none"
      inert={!wideSidebar.open}
      aria-hidden={!wideSidebar.open || undefined}
      data-wide-sidebar
    >
      <div class="flex h-12 shrink-0 items-center gap-1 ps-3.5 pe-1 pt-1">
        <a href="/" class="flex min-w-0 flex-1 items-center">
          <Wordmark size="small" bright />
        </a>
        <WideSidebarToggle />
      </div>
      <div class="shrink-0 px-2.5 pb-2">
        <GlobalSearch
          value={ui.searchQuery}
          onInput={search.inputEvent}
          onSubmit={search.submitEvent}
          onClear={search.clear}
          onFocus={search.focus}
        />
      </div>
      <div class="min-h-0 flex-1">
        {#if settingsRoute}
          <SettingsSidebar />
        {:else}
          <LeftSidebar />
        {/if}
      </div>
      <SidebarFoot />
    </div>
  </div>
  <main data-page class="flex h-full min-w-0 flex-1 flex-col bg-light">
    {@render children()}
  </main>
  <JobsDock />
</div>
<AccountDrawer />
