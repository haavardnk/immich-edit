<script lang="ts">
  import { page } from '$app/state';
  import type { Snippet } from 'svelte';
  import TopBar from '#lib/components/shell/desktop/TopBar.svelte';
  import LeftSidebar from '#lib/components/shell/LeftSidebar.svelte';
  import SettingsSidebar from '#lib/components/shell/SettingsSidebar.svelte';
  import { navSidebar } from '#lib/components/shell/desktop/navSidebar.svelte.ts';
  import { AppShell, AppShellBar, AppShellSidebar } from '@immich/ui';

  let { children }: { children: Snippet } = $props();

  const settingsRoute = $derived(page.url.pathname.startsWith('/settings'));

  $effect(() => {
    if (page.url.pathname && navSidebar.narrow) navSidebar.open = false;
  });
</script>

<AppShell class="app-frame">
  <AppShellBar class="min-h-18 border-b border-hairline bg-light-50 py-1">
    <TopBar onToggleSidebar={navSidebar.toggle} />
  </AppShellBar>
  <AppShellSidebar bind:open={navSidebar.open} border={false} class="bg-light-50">
    {#if navSidebar.open}
      {#if settingsRoute}
        <SettingsSidebar />
      {:else}
        <LeftSidebar />
      {/if}
    {/if}
  </AppShellSidebar>
  <main class="flex h-full min-w-0 flex-col bg-light">
    {@render children()}
  </main>
</AppShell>
