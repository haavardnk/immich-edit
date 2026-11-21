<script lang="ts">
  import type { PageParent } from '#lib/components/shell/pageParent.ts';
  import TouchAppBar from '#lib/components/shell/touch/TouchAppBar.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    title,
    parent,
    count,
    badge,
    actions,
    titleActions,
    touchActions,
    compactActions,
    bar,
    toolbar,
    selecting
  }: {
    title: string;
    parent?: PageParent;
    count?: string;
    badge?: string;
    actions?: Snippet;
    titleActions?: Snippet;
    touchActions?: Snippet;
    compactActions?: Snippet;
    bar?: Snippet;
    toolbar?: Snippet;
    selecting?: Snippet;
  } = $props();

  const headerActions = $derived(layout.mode === 'touch' ? (touchActions ?? actions) : actions);
  const appBarActions = $derived(compactActions ?? touchActions);
</script>

{#if selecting && layout.mode === 'touch'}
  <header class="sticky flex h-13 shrink-0 items-center gap-1 bg-light px-1.5" data-app-bar>
    <h1 class="sr-only">{title}</h1>
    {@render selecting()}
  </header>
{:else if layout.compact}
  <TouchAppBar {title} {count} actions={appBarActions} {titleActions} {bar} />
  {#if toolbar}
    <div class="flex shrink-0 flex-wrap items-center gap-2 px-4 py-1 sm:px-5">
      {@render toolbar()}
    </div>
  {/if}
{:else}
  <header
    class="flex h-12 shrink-0 items-center gap-3 border-b border-hairline bg-light px-3 sm:px-5"
  >
    <div class="flex min-w-0 items-center gap-2 {toolbar ? 'shrink-0' : 'flex-1'}">
      {#if parent}
        <a
          href={parent.href}
          class="shrink-0 rounded text-sm text-dark/65 outline-none hover:text-primary focus-visible:ring-2 focus-visible:ring-primary"
        >
          {parent.label}
        </a>
        <Icon
          icon={mdiChevronRight}
          size="1em"
          class="-mx-1 shrink-0 text-dark/45"
          aria-hidden="true"
        />
      {/if}
      <h1 class="truncate text-sm font-semibold text-light-950">{title}</h1>
      {#if count}
        <p
          class="shrink-0 rounded-full border border-hairline px-2 text-caption leading-5 text-dark/65 tabular-nums"
        >
          {#if badge === undefined}
            {count}
          {:else}
            <span aria-hidden="true">{badge}</span>
            <span class="sr-only">{count}</span>
          {/if}
        </p>
      {/if}
      {@render titleActions?.()}
    </div>
    {#if toolbar}
      <div class="flex min-w-0 flex-1 items-center gap-3">
        {@render toolbar()}
      </div>
    {/if}
    {#if headerActions}
      <div class="flex shrink-0 items-center gap-1.5">
        {@render headerActions()}
      </div>
    {/if}
  </header>
{/if}
