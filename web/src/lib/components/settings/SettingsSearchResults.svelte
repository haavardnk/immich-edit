<script lang="ts">
  import { page } from '$app/state';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import type { SettingsHit } from '#lib/settings/searchSettings.ts';
  import type { SettingsPage } from '#lib/settings/pages.ts';
  import { Icon, Link } from '@immich/ui';
  import { on } from 'svelte/events';

  let {
    hits,
    pages,
    query,
    onpick,
    onkeydown
  }: {
    hits: readonly SettingsHit[];
    pages: readonly SettingsPage[];
    query: string;
    onpick: (hit: SettingsHit) => void;
    onkeydown: (event: KeyboardEvent) => void;
  } = $props();

  const groups = $derived(
    pages
      .map((entry) => ({ page: entry, hits: hits.filter((hit) => hit.page.id === entry.id) }))
      .filter((group) => group.hits.length > 0)
  );
  const current = $derived(`${page.url.pathname}${page.url.hash}`);
</script>

<div
  class="flex flex-col gap-2 py-1"
  role="region"
  aria-label="Settings search results"
  {@attach (node) => on(node, 'keydown', onkeydown)}
>
  {#each groups as group (group.page.id)}
    <section class="flex flex-col" aria-label="{group.page.title} results">
      <div
        class="flex items-baseline justify-between px-3 pt-1 pb-0.5 text-[10.5px] font-semibold tracking-wide text-dark/80 uppercase touch:px-4 touch:text-[11px]"
      >
        <h2>{group.page.title}</h2>
        <span class="tabular-nums">{group.hits.length}</span>
      </div>
      {#each group.hits as hit (hit.entry.id)}
        <Link
          href={hit.href}
          underline={false}
          data-settings-hit
          aria-current={current === hit.href ? 'page' : undefined}
          onclick={() => onpick(hit)}
          class="flex min-w-0 items-start gap-3 px-3 py-1.5 text-dark transition-colors outline-none hover:bg-light-950/6 focus-visible:bg-primary/10 aria-[current=page]:bg-primary/10 touch:px-4 touch:py-2.5"
        >
          <Icon
            icon={hit.page.icon}
            size="16px"
            class="mt-0.5 shrink-0 text-dark/80"
            aria-hidden="true"
          />
          <span class="flex min-w-0 flex-col">
            <span class="text-[13px] touch:text-[15px]">
              <HighlightedText text={hit.entry.title} {query} />
            </span>
            <span class="truncate text-[11.5px] text-dark/80 touch:text-[13px]">
              <HighlightedText text={hit.entry.detail} {query} />
            </span>
          </span>
        </Link>
      {/each}
    </section>
  {/each}
</div>
