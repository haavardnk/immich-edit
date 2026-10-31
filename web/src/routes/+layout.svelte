<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { setupStatus } from '#lib/api/auth.ts';
  import { isBackendDown } from '#lib/api/client.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { syncFilterUrl } from '#lib/browse/filterUrl.svelte.ts';
  import { installFocusPolicy } from '#lib/utils/pointerFocus.ts';
  import BackendUnavailable from '#lib/components/shell/BackendUnavailable.svelte';
  import Shell from '#lib/components/shell/Shell.svelte';
  import Toasts from '#lib/components/shell/Toasts.svelte';
  import { setLocale, TooltipProvider } from '@immich/ui';

  let { children } = $props();

  setLocale(navigator.language);

  let bootState = $state<'loading' | 'ready' | 'unreachable'>('loading');
  let configured = $state(false);

  const bare = $derived(page.url.pathname === '/login' || page.url.pathname === '/setup');

  async function boot(): Promise<void> {
    bootState = 'loading';
    try {
      const st = await setupStatus();
      configured = st.configured;
      if (configured && (await session.load())) await immichCapabilities.load();
      bootState = 'ready';
    } catch (err: unknown) {
      bootState = isBackendDown(err) ? 'unreachable' : 'ready';
    }
  }

  onMount(boot);
  onMount(installFocusPolicy);

  $effect(() => {
    if (bootState !== 'ready') return;
    const path = page.url.pathname;
    if (session.user) {
      if (path === '/login' || path === '/setup') void goto('/', { replace: true });
      return;
    }
    if (!configured) {
      if (path !== '/setup') void goto('/setup', { replace: true });
      return;
    }
    if (path !== '/login') {
      const next = encodeURIComponent(path + page.url.search);
      void goto(`/login?next=${next}`, { replace: true });
    }
  });

  syncFilterUrl(() => bootState === 'ready');
</script>

<TooltipProvider>
  {#if bootState === 'loading'}
    <div class="h-screen w-screen bg-light text-dark flex items-center justify-center">
      <div
        class="h-8 w-8 rounded-full border-2 border-white/20 border-t-primary animate-spin"
      ></div>
    </div>
  {:else if bootState === 'unreachable'}
    <div class="h-screen w-screen bg-light text-dark">
      <BackendUnavailable retry={boot} />
    </div>
  {:else if bare}
    <div class="h-screen w-screen bg-light text-dark">
      {@render children()}
    </div>
  {:else if session.user}
    <Shell>
      {@render children()}
    </Shell>
  {:else}
    <div class="h-screen w-screen bg-light text-dark flex items-center justify-center">
      <div
        class="h-8 w-8 rounded-full border-2 border-white/20 border-t-primary animate-spin"
      ></div>
    </div>
  {/if}
  <Toasts />
</TooltipProvider>
