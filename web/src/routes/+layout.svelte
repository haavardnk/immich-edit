<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { afterNavigate, goto } from '$app/navigation';
  import { page } from '$app/state';
  import { setupStatus } from '#lib/api/auth.ts';
  import { isBackendDown } from '#lib/api/client.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { navHistory } from '#lib/stores/navHistory.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { syncFilterUrl } from '#lib/browse/filterUrl.svelte.ts';
  import { installFocusPolicy } from '#lib/utils/pointerFocus.ts';
  import { listenForCommands } from '#lib/commands/listen.ts';
  import { installLayout, layout } from '#lib/device/layout.svelte.ts';
  import { COMPACT_PANES, compare, MAX_PANES } from '#lib/stores/compare.svelte.ts';
  import { installPointerKind } from '#lib/device/pointerKind.svelte.ts';
  import { showLayout } from '#lib/device/document.ts';
  import '#lib/pwa/install.svelte.ts';
  import { checkForUpdate, installServiceWorker } from '#lib/pwa/serviceWorker.ts';
  import BackendUnavailable from '#lib/components/shell/BackendUnavailable.svelte';
  import Shell from '#lib/components/shell/Shell.svelte';
  import Toasts from '#lib/components/shell/Toasts.svelte';
  import AppTooltipProvider from '#lib/components/AppTooltipProvider.svelte';
  import { setLocale } from '@immich/ui';

  let { children } = $props();

  setLocale(navigator.language);
  appearance.apply();

  let bootState = $state<'loading' | 'ready' | 'unreachable'>('loading');
  let configured = $state(false);

  const bare = $derived(page.url.pathname === '/login' || page.url.pathname === '/setup');

  async function boot(): Promise<void> {
    bootState = 'loading';
    try {
      const st = await setupStatus();
      configured = st.configured;
      if (configured && (await session.load())) {
        void appearance.load();
        await immichCapabilities.load();
      }
      bootState = 'ready';
    } catch (err: unknown) {
      bootState = isBackendDown(err) ? 'unreachable' : 'ready';
    }
  }

  onMount(boot);
  onMount(installFocusPolicy);
  onMount(listenForCommands);
  onMount(installLayout);
  onMount(() => compare.limitBy(() => (layout.compact ? COMPACT_PANES : MAX_PANES)));
  onMount(installPointerKind);
  onMount(() => installServiceWorker(() => editor.saving || editor.saveError !== null));
  afterNavigate(checkForUpdate);
  afterNavigate(navHistory.record);

  $effect(() => appearance.showScheme(appearance.scheme));
  $effect(() => showLayout(layout.mode, layout.wide));

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

<AppTooltipProvider>
  {#if bootState === 'loading'}
    <div class="app-frame flex items-center justify-center">
      <div
        class="h-8 w-8 rounded-full border-2 border-light-950/20 border-t-primary animate-spin"
      ></div>
    </div>
  {:else if bootState === 'unreachable'}
    <div class="app-frame">
      <BackendUnavailable retry={boot} />
    </div>
  {:else if bare}
    <div class="app-frame">
      {@render children()}
    </div>
  {:else if session.user}
    <Shell>
      {@render children()}
    </Shell>
  {:else}
    <div class="app-frame flex items-center justify-center">
      <div
        class="h-8 w-8 rounded-full border-2 border-light-950/20 border-t-primary animate-spin"
      ></div>
    </div>
  {/if}
  <Toasts />
</AppTooltipProvider>
