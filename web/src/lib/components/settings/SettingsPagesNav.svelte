<script lang="ts">
  import { page } from '$app/state';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { settingsHref, visiblePages } from '#lib/settings/pages.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Icon, Link } from '@immich/ui';
  import { mdiChevronRight } from '@mdi/js';

  const pages = $derived(visiblePages(session.isAdmin));
  const current = $derived(page.params.id ?? null);
  const pushes = $derived(layout.compact);
</script>

<nav
  aria-label="Settings pages"
  data-sveltekit-replacestate={pushes ? undefined : ''}
  class="flex flex-col gap-0.5 touch:gap-0 {pushes
    ? 'divide-y divide-hairline overflow-hidden rounded-2xl bg-light-100'
    : ''}"
>
  {#each pages as entry (entry.id)}
    <Link
      href={settingsHref(entry.id)}
      underline={false}
      aria-current={current === entry.id ? 'page' : undefined}
      class="flex h-10 items-center gap-3 rounded-e-full ps-3.5 pe-3 text-sm text-dark transition-colors hover:bg-light-950/6 aria-[current=page]:bg-primary/10 aria-[current=page]:text-primary touch:h-12 touch:rounded-xl touch:text-body {pushes
        ? 'touch:h-13 touch:rounded-none touch:px-4'
        : ''}"
    >
      <Icon icon={entry.icon} size="20px" class="shrink-0" aria-hidden="true" />
      <span class="min-w-0 flex-1 truncate">{entry.title}</span>
      {#if pushes}
        <Icon icon={mdiChevronRight} size="20px" class="shrink-0 text-dark/45" aria-hidden="true" />
      {/if}
    </Link>
  {/each}
</nav>
