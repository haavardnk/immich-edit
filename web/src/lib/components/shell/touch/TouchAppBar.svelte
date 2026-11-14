<script lang="ts">
  import { page } from '$app/state';
  import BackButton from '#lib/components/shell/BackButton.svelte';
  import AvatarButton from '#lib/components/shell/account/AvatarButton.svelte';
  import { isTabRoot } from '#lib/components/shell/touch/navTabs.ts';
  import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
  import type { Snippet } from 'svelte';

  let {
    title,
    count,
    actions,
    bar
  }: {
    title: string;
    count?: string;
    actions?: Snippet;
    bar?: Snippet;
  } = $props();
</script>

<header
  class="sticky flex h-13 shrink-0 items-center gap-1 bg-light px-1.5 motion-safe:transition-[margin] motion-safe:duration-200 {touchBars.hidden
    ? '-mt-13'
    : ''}"
  inert={touchBars.hidden}
  data-app-bar
>
  {#if bar}
    <h1 class="sr-only">{title}</h1>
    {@render bar()}
  {:else}
    {#if isTabRoot(page.url.pathname)}
      <AvatarButton />
    {:else}
      <BackButton />
    {/if}
    <div class="min-w-0 flex-1 ps-1 leading-tight">
      <h1 class="truncate text-xl font-semibold text-light-950">{title}</h1>
      {#if count}
        <p class="truncate text-xs text-dark/65">{count}</p>
      {/if}
    </div>
  {/if}
  {@render actions?.()}
</header>
