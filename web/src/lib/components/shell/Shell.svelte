<script lang="ts">
  import type { Snippet } from 'svelte';
  import TopBar from './TopBar.svelte';
  import LeftSidebar from './LeftSidebar.svelte';
  import RightSidebar from './RightSidebar.svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div class="h-screen w-screen flex flex-col bg-base-100 text-base-content overflow-hidden">
  <TopBar />
  <div class="flex-1 flex min-h-0">
    <LeftSidebar />
    <main class="flex-1 min-w-0 min-h-0 flex flex-col bg-base-200">
      {@render children()}
    </main>
    <RightSidebar />
  </div>
</div>
