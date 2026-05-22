<script lang="ts">
  import type { Snippet } from 'svelte';
  import TopBar from './TopBar.svelte';
  import LeftSidebar from './LeftSidebar.svelte';
  import RightSidebar from './RightSidebar.svelte';
  import Filmstrip from './Filmstrip.svelte';
  import Toasts from './Toasts.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div class="h-screen w-screen flex flex-col bg-immich-dark-bg text-immich-dark-fg overflow-hidden">
  {#if !ui.fullscreen}
    <TopBar />
  {/if}
  <div class="flex-1 flex min-h-0">
    {#if !ui.fullscreen}
      <LeftSidebar />
    {/if}
    <div class="flex-1 min-w-0 min-h-0 flex flex-col">
      <main class="flex-1 min-h-0 flex flex-col bg-immich-dark-bg">
        {@render children()}
      </main>
      {#if editor.assetId && !ui.fullscreen}
        <Filmstrip />
      {/if}
    </div>
    {#if !ui.fullscreen}
      <RightSidebar />
    {/if}
  </div>
  <Toasts />
</div>
