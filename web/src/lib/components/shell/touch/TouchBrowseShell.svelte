<script lang="ts">
  import { page } from '$app/state';
  import AccountDrawer from '#lib/components/shell/touch/AccountDrawer.svelte';
  import BottomNav from '#lib/components/shell/touch/BottomNav.svelte';
  import JobsPill from '#lib/components/shell/touch/JobsPill.svelte';
  import { accountDrawer } from '#lib/components/shell/touch/accountDrawer.svelte.ts';
  import { hideBarsOnScroll } from '#lib/components/shell/touch/barsHide.ts';
  import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { onDestroy, type Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  const TAB_BAR_BOTTOM = 'calc(3.5rem + 1px + var(--safe-bottom))';

  const hideBars = hideBarsOnScroll({
    enabled: () => layout.landscape,
    route: () => page.url.pathname,
    onChange: touchBars.set
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      '--chrome-bottom',
      touchBars.hidden ? 'var(--safe-bottom)' : TAB_BAR_BOTTOM
    );
  });

  onDestroy(() => {
    accountDrawer.close();
    document.documentElement.style.removeProperty('--chrome-bottom');
  });
</script>

<div class="app-frame flex flex-col overflow-hidden pb-0!" {@attach hideBars}>
  <main class="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-light">
    {@render children()}
  </main>
  <BottomNav />
</div>
<JobsPill />
<AccountDrawer />
