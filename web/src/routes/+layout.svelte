<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';
  import Shell from '$lib/components/shell/Shell.svelte';
  import Toasts from '$lib/components/shell/Toasts.svelte';

  let { children } = $props();
  const bare = $derived(page.url.pathname === '/login');
</script>

{#if bare}
  <div class="h-screen w-screen bg-immich-dark-bg text-immich-dark-fg">
    {@render children()}
  </div>
  <Toasts />
{:else}
  <Shell>
    {@render children()}
  </Shell>
{/if}
