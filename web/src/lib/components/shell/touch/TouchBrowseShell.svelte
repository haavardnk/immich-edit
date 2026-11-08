<script lang="ts">
  import { page } from '$app/state';
  import AccountDrawer from '#lib/components/shell/account/AccountDrawer.svelte';
  import BottomNav from '#lib/components/shell/touch/BottomNav.svelte';
  import JobsPill from '#lib/components/shell/touch/JobsPill.svelte';
  import { accountDrawer } from '#lib/components/shell/account/accountDrawer.svelte.ts';
  import { hideBarsOnScroll } from '#lib/components/shell/touch/barsHide.ts';
  import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
  import { TAB_BAR_BOTTOM } from '#lib/components/shell/touch/touchChrome.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { provideTileOpen } from '#lib/browse/tileOpen.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { onDestroy, type Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  provideTileOpen('loupe');

  const selecting = $derived(bulkScope.selecting);

  const hideBars = hideBarsOnScroll({
    enabled: () => layout.landscape && !selecting,
    route: () => page.url.pathname,
    onChange: touchBars.set
  });

  $effect(() => {
    const root = document.documentElement.style;
    root.setProperty('--chrome-bottom', touchBars.hidden ? 'var(--safe-bottom)' : TAB_BAR_BOTTOM);
    root.setProperty('--chrome-inset', touchBars.hidden ? 'var(--safe-bottom)' : '0px');
  });

  onDestroy(() => {
    accountDrawer.close();
    document.documentElement.style.removeProperty('--chrome-bottom');
    document.documentElement.style.removeProperty('--chrome-inset');
  });
</script>

<div class="app-frame flex flex-col overflow-hidden pb-0!" {@attach hideBars}>
  <main data-page class="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-light">
    {@render children()}
  </main>
  {#if !selecting}
    <BottomNav />
  {/if}
</div>
<JobsPill />
<AccountDrawer />
