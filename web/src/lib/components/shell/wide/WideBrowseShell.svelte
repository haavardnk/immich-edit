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
  import { SearchInput } from '#lib/browse/searchInput.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let { children }: { children: Snippet } = $props();

  const settingsRoute = $derived(page.url.pathname.startsWith('/settings'));

  const search = new SearchInput('/photos');

  onDestroy(() => accountDrawer.close());
</script>

<div class="app-frame flex overflow-hidden">
  <div class="flex h-full w-70 shrink-0 flex-col border-e border-hairline bg-light-50 select-none">
    <a href="/" class="flex h-12 shrink-0 items-center px-3.5 pt-1">
      <Wordmark size="small" bright />
    </a>
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
  <main class="flex h-full min-w-0 flex-1 flex-col bg-light">
    {@render children()}
  </main>
</div>
<AccountDrawer />
