<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { setupStatus } from '$lib/api/auth';
  import { isBackendDown } from '$lib/api/client';
  import { session } from '$lib/stores/session.svelte';
  import BackendUnavailable from '$lib/components/shell/BackendUnavailable.svelte';
  import Shell from '$lib/components/shell/Shell.svelte';
  import Toasts from '$lib/components/shell/Toasts.svelte';

  let { children } = $props();

  let bootState = $state<'loading' | 'ready' | 'unreachable'>('loading');
  let configured = $state(false);

  const bare = $derived(page.url.pathname === '/login' || page.url.pathname === '/setup');

  async function boot(): Promise<void> {
    bootState = 'loading';
    try {
      const st = await setupStatus();
      configured = st.configured;
      if (configured) await session.load();
      bootState = 'ready';
    } catch (err: unknown) {
      bootState = isBackendDown(err) ? 'unreachable' : 'ready';
    }
  }

  onMount(boot);

  $effect(() => {
    if (bootState !== 'ready') return;
    const path = page.url.pathname;
    if (session.user) {
      if (path === '/login' || path === '/setup') void goto('/', { replaceState: true });
      return;
    }
    if (!configured) {
      if (path !== '/setup') void goto('/setup', { replaceState: true });
      return;
    }
    if (path !== '/login') {
      const next = encodeURIComponent(path + page.url.search);
      void goto(`/login?next=${next}`, { replaceState: true });
    }
  });
</script>

{#if bootState === 'loading'}
  <div class="h-screen w-screen bg-immich-dark-bg text-immich-dark-fg flex items-center justify-center">
    <div class="h-8 w-8 rounded-full border-2 border-white/20 border-t-immich-primary animate-spin"></div>
  </div>
{:else if bootState === 'unreachable'}
  <div class="h-screen w-screen bg-immich-dark-bg text-immich-dark-fg">
    <BackendUnavailable retry={boot} />
  </div>
{:else if bare}
  <div class="h-screen w-screen bg-immich-dark-bg text-immich-dark-fg">
    {@render children()}
  </div>
  <Toasts />
{:else if session.user}
  <Shell>
    {@render children()}
  </Shell>
{:else}
  <div class="h-screen w-screen bg-immich-dark-bg text-immich-dark-fg flex items-center justify-center">
    <div class="h-8 w-8 rounded-full border-2 border-white/20 border-t-immich-primary animate-spin"></div>
  </div>
{/if}
