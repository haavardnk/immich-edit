<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import { page } from '$app/state';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { wideSidebar } from '#lib/components/shell/wide/wideSidebar.svelte.ts';
  import { findPage } from '#lib/settings/pages.ts';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  const current = $derived(findPage(page.params.id));

  let scroller = $state<HTMLElement | null>(null);

  afterNavigate(({ from, to }) => {
    if (scroller && from?.url.pathname !== to?.url.pathname) scroller.scrollTop = 0;
  });
</script>

<div class="flex h-full min-h-0 flex-col bg-light-50">
  {#if layout.compact || (layout.wide && !wideSidebar.open)}
    <PageHeader title={current?.title ?? 'Settings'} />
  {/if}
  <div bind:this={scroller} class="min-h-0 flex-1 overflow-y-auto">
    <div
      class="w-full max-w-3xl px-4 py-4 sm:px-10 sm:py-7 {layout.compact
        ? 'mx-auto'
        : ''} wide:px-8"
    >
      {@render children()}
    </div>
  </div>
</div>
