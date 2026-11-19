<script lang="ts">
  import { page } from '$app/state';
  import { PAGE_BODIES } from '#lib/components/settings/pages/pageBodies.ts';
  import { revealAnchor } from '#lib/settings/revealAnchor.ts';
  import type { SettingsPage } from '#lib/settings/pages.ts';

  let { entry, heading }: { entry: SettingsPage; heading: boolean } = $props();

  let root = $state<HTMLElement | null>(null);

  const Body = $derived(PAGE_BODIES[entry.id]);
  const anchor = $derived(page.url.hash.slice(1));

  $effect(() => {
    if (root && anchor) return revealAnchor(root, anchor);
  });
</script>

<article
  bind:this={root}
  aria-labelledby={heading ? 'settings-page-title' : undefined}
  class="flex flex-col gap-6"
>
  {#if heading}
    <h1 id="settings-page-title" class="text-xl font-semibold text-light-950">{entry.title}</h1>
  {/if}
  <Body />
</article>
