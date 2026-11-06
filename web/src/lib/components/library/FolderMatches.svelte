<script lang="ts">
  import { page } from '$app/state';
  import type { FolderNode } from '#lib/stores/library.svelte.ts';
  import { activeFolderPath, folderHref, parentFolder } from '#lib/browse/folders.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { folderLinkClass } from '#lib/components/library/folderLink.ts';
  import { Icon } from '@immich/ui';
  import { mdiFolderOutline } from '@mdi/js';

  let { folders, query }: { folders: readonly FolderNode[]; query: string } = $props();

  const activePath = $derived(activeFolderPath(page.url));
</script>

{#if folders.length > 0}
  <div class="flex flex-col gap-0.5 p-1">
    {#each folders as node (node.path)}
      {@const parent = parentFolder(node.path)}
      <TooltipLink
        tooltip={node.path}
        href={folderHref(node.path)}
        aria-current={node.path === activePath ? 'page' : undefined}
        class="gap-2 ps-3 touch:min-h-11 {folderLinkClass(node.path === activePath)}"
      >
        <Icon icon={mdiFolderOutline} size="14px" class="opacity-40 flex-none" aria-hidden="true" />
        <span class="flex min-w-0 flex-1 flex-col pe-2 leading-tight">
          <span data-truncate class="truncate text-[13px]">
            <HighlightedText text={node.name} {query} />
          </span>
          {#if parent}
            <span data-truncate class="truncate text-[10px] text-muted">{parent}</span>
          {/if}
        </span>
      </TooltipLink>
    {/each}
  </div>
{/if}
