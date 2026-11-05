<script lang="ts">
  import { folderCrumbs, folderHref } from '#lib/browse/folders.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
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
  <TooltipLink
    tooltip={name}
    {href}
    aria-current={current ? 'page' : undefined}
    class="group flex h-8 shrink-0 items-center outline-none touch:h-11"
  >
    <span
      class="flex h-8 max-w-48 items-center gap-1.5 rounded-lg px-2 text-xs whitespace-nowrap group-focus-visible:ring-2 group-focus-visible:ring-primary touch:text-[13.5px] {current
        ? 'bg-light-100 font-medium text-light-950'
        : 'text-dark/65 group-hover:bg-subtle'}"
    >
      {#if icon}
        <Icon {icon} size="16px" class="shrink-0" aria-hidden="true" />
      {/if}
      <span data-truncate class="truncate">{name}</span>
    </span>
  </TooltipLink>
{/snippet}

<nav
  aria-label="Folder path"
  class="flex shrink-0 items-center gap-0.5 overflow-x-auto border-b border-hairline bg-light px-2 py-1.5 scrollbar-hidden sm:px-4 touch:border-b-0 touch:px-3 touch:py-0"
  {@attach keepCurrentInView}
>
  {@render crumb('Folders', folderHref(''), crumbs.length === 0, mdiFolderOutline)}
  {#each crumbs as entry, index (entry.path)}
    <Icon icon={mdiChevronRight} size="14px" class="shrink-0 text-dark/40" aria-hidden="true" />
    {@render crumb(entry.name, folderHref(entry.path), index === crumbs.length - 1)}
  {/each}
</nav>
