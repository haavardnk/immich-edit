<script lang="ts">
  import { page } from '$app/state';
  import type { FolderNode } from '$lib/stores/library.svelte';
  import FolderTree from './FolderTree.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiFolderOutline, mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let { nodes, depth = 0 }: { nodes: FolderNode[]; depth?: number } = $props();

  let expanded = $state(new Set<string>());
  const activePath = $derived(
    page.url.pathname === '/folders' ? page.url.searchParams.get('path') : null
  );

  function toggle(path: string): void {
    if (expanded.has(path)) {
      expanded.delete(path);
    } else {
      expanded.add(path);
    }
    expanded = new Set(expanded);
  }
</script>

{#if nodes.length === 0}
  {#if depth === 0}
    <div class="px-5 py-3 text-xs text-muted" role="status">No folders</div>
  {/if}
{:else}
  <div class="flex flex-col gap-0.5" class:p-1={depth === 0} class:pl-4={depth > 0}>
    {#each nodes as node (node.path)}
      {#if node.children.length > 0}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="w-full justify-start gap-1.5 rounded-e-full px-2 py-1 text-left"
          aria-expanded={expanded.has(node.path)}
          onclick={() => toggle(node.path)}
        >
          <Icon
            icon={expanded.has(node.path) ? mdiChevronDown : mdiChevronRight}
            size="14px"
            class="opacity-40 flex-none"
            aria-hidden="true"
          />
          <Icon
            icon={mdiFolderOutline}
            size="14px"
            class="opacity-40 flex-none"
            aria-hidden="true"
          />
          <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
        </Button>
        {#if expanded.has(node.path)}
          <FolderTree nodes={node.children} depth={depth + 1} />
        {/if}
      {:else}
        <a
          href={`/folders?path=${encodeURIComponent(node.path)}`}
          aria-current={node.path === activePath ? 'page' : undefined}
          class="flex items-center gap-1.5 rounded-e-full py-1 ps-7 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {node.path ===
          activePath
            ? 'bg-primary/15 text-primary'
            : 'hover:bg-subtle hover:text-primary'}"
        >
          <Icon
            icon={mdiFolderOutline}
            size="14px"
            class="opacity-40 flex-none"
            aria-hidden="true"
          />
          <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
        </a>
      {/if}
    {/each}
  </div>
{/if}
