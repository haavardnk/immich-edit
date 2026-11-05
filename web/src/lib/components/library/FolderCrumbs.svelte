<script lang="ts">
  import { folderCrumbs, folderHref } from '#lib/browse/folders.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight, mdiFolderOutline } from '@mdi/js';
  import type { Attachment } from 'svelte/attachments';

  let { path }: { path: string } = $props();

  const crumbs = $derived(folderCrumbs(path));

  const keepCurrentInView: Attachment<HTMLElement> = (element) => {
    if (crumbs.length > 0) element.scrollLeft = element.scrollWidth;
  };
</script>

{#snippet crumb(name: string, href: string, current: boolean, icon?: string)}
  <a
    {href}
    aria-current={current ? 'page' : undefined}
    class="flex h-8 shrink-0 items-center touch:h-11 gap-1.5 rounded-full px-2.5 text-xs whitespace-nowrap outline-none focus-visible:ring-2 focus-visible:ring-primary {current
      ? 'bg-primary/15 font-semibold text-primary'
      : 'text-dark/80 hover:bg-subtle'}"
  >
    {#if icon}
      <Icon {icon} size="16px" aria-hidden="true" />
    {/if}
    {name}
  </a>
{/snippet}

<nav
  aria-label="Folder path"
  class="flex shrink-0 items-center gap-0.5 overflow-x-auto border-b border-hairline bg-light px-2 py-1.5 scrollbar-hidden sm:px-4"
  {@attach keepCurrentInView}
>
  {@render crumb('Folders', folderHref(''), crumbs.length === 0, mdiFolderOutline)}
  {#each crumbs as entry, index (entry.path)}
    <Icon icon={mdiChevronRight} size="14px" class="shrink-0 text-dark/40" aria-hidden="true" />
    {@render crumb(entry.name, folderHref(entry.path), index === crumbs.length - 1)}
  {/each}
</nav>
