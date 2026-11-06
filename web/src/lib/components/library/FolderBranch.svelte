<script lang="ts">
  import { page } from '$app/state';
  import type { FolderNode } from '#lib/stores/library.svelte.ts';
  import { activeFolderPath, folderHref } from '#lib/browse/folders.ts';
  import FolderBranch from './FolderBranch.svelte';
  import { folderLinkClass } from '#lib/components/library/folderLink.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiFolderOutline, mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let {
    nodes,
    depth = 0,
    expanded,
    onToggle
  }: {
    nodes: readonly FolderNode[];
    depth?: number;
    expanded: ReadonlySet<string>;
    onToggle: (path: string) => void;
  } = $props();

  const activePath = $derived(activeFolderPath(page.url));
</script>

<div class="flex flex-col gap-0.5" class:p-1={depth === 0} class:pl-4={depth > 0}>
  {#each nodes as node (node.path)}
    {#if node.children.length > 0}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="w-full justify-start gap-1.5 rounded-e-full px-2 py-1 text-left"
        aria-expanded={expanded.has(node.path)}
        onclick={() => onToggle(node.path)}
      >
        <Icon
          icon={expanded.has(node.path) ? mdiChevronDown : mdiChevronRight}
          size="14px"
          class="opacity-40 flex-none"
          aria-hidden="true"
        />
        <Icon icon={mdiFolderOutline} size="14px" class="opacity-40 flex-none" aria-hidden="true" />
        <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
      </Button>
      {#if expanded.has(node.path)}
        <FolderBranch nodes={node.children} depth={depth + 1} {expanded} {onToggle} />
      {/if}
    {:else}
      <a
        href={folderHref(node.path)}
        aria-current={node.path === activePath ? 'page' : undefined}
        class="gap-1.5 ps-7 {folderLinkClass(node.path === activePath)}"
      >
        <Icon icon={mdiFolderOutline} size="14px" class="opacity-40 flex-none" aria-hidden="true" />
        <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
      </a>
    {/if}
  {/each}
</div>
