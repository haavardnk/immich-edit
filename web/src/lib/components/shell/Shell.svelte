<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import type { Snippet } from 'svelte';
  import TopBar from './TopBar.svelte';
  import LeftSidebar from './LeftSidebar.svelte';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import RightSidebar from './RightSidebar.svelte';
  import Filmstrip from './Filmstrip.svelte';
  import KeybindsHelp from './KeybindsHelp.svelte';
  import CopyDialog from './CopyDialog.svelte';
  import MetadataConsentDialog from './MetadataConsentDialog.svelte';
  import JobsDrawer from '$lib/components/jobs/JobsDrawer.svelte';
  import EditorToolRail from '$lib/components/editor/EditorToolRail.svelte';
  import Loupe from '$lib/components/browse/Loupe.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { jobs } from '$lib/stores/jobs.svelte';
  import { AppShell, AppShellBar, AppShellSidebar } from '@immich/ui';
  import { MediaQuery } from 'svelte/reactivity';

  let { children }: { children: Snippet } = $props();

  const editorRoute = $derived(page.url.pathname.startsWith('/assets/'));
  const settingsRoute = $derived(page.url.pathname.startsWith('/settings'));
  const narrowViewport = new MediaQuery('max-width: 1023px');
  let sidebarOpen = $derived(!narrowViewport.current);

  function toggleSidebar(): void {
    sidebarOpen = !sidebarOpen;
  }

  onMount(() => {
    void editedThumbs.loadOnce();
    void jobs.load();
  });

  $effect(() => {
    if (page.url.pathname && narrowViewport.current) sidebarOpen = false;
  });

  $effect(() => {
    document.documentElement.classList.toggle('grey-canvas', ui.greyCanvas);
  });
</script>

<div class="contents" inert={browseView.loupeId !== null}>
  {#if editorRoute}
    <div class="flex h-screen w-screen overflow-hidden bg-black text-dark">
      <div class="relative flex min-h-0 min-w-0 flex-1">
        <div class="flex min-h-0 min-w-0 flex-1 flex-col">
          <main class="flex-1 min-h-0 flex flex-col bg-light">
            {@render children()}
          </main>
          {#if editor.assetId && !ui.fullscreen}
            <Filmstrip resizable collapsed={ui.editorFilmstripCollapsed} />
          {/if}
        </div>
        {#if !ui.fullscreen}
          <RightSidebar />
          <EditorToolRail />
        {/if}
      </div>
    </div>
  {:else}
    <AppShell class="h-screen w-screen bg-light text-dark">
      <AppShellBar class="min-h-18 border-b border-hairline bg-neutral-950 py-1">
        <TopBar onToggleSidebar={toggleSidebar} />
      </AppShellBar>
      <AppShellSidebar bind:open={sidebarOpen} border={false} class="bg-neutral-950">
        {#if sidebarOpen}
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
  {/if}
</div>

<KeybindsHelp />
<CopyDialog />
<MetadataConsentDialog />
<JobsDrawer />
{#if browseView.loupeId}
  <Loupe />
{/if}
