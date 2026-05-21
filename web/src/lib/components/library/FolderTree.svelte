<script lang="ts">
  import type { FolderNode } from '$lib/stores/library.svelte';
  import FolderTree from './FolderTree.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiFolderOutline, mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let { nodes, depth = 0 }: { nodes: FolderNode[]; depth?: number } = $props();

  let expanded = $state(new Set<string>());

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
    <div class="p-3 text-xs text-immich-dark-fg/30">no folders</div>
  {/if}
{:else}
  <div class="flex flex-col gap-0.5" class:p-1={depth === 0} class:pl-4={depth > 0}>
    {#each nodes as node (node.path)}
      {#if node.children.length > 0}
        <button
          class="flex items-center gap-1.5 py-1 px-2 rounded-lg hover:bg-white/5 transition-colors text-left w-full"
          onclick={() => toggle(node.path)}
        >
          <Icon path={expanded.has(node.path) ? mdiChevronDown : mdiChevronRight} size={14} class="opacity-40 flex-none" />
          <Icon path={mdiFolderOutline} size={14} class="opacity-40 flex-none" />
          <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
        </button>
        {#if expanded.has(node.path)}
          <FolderTree nodes={node.children} depth={depth + 1} />
        {/if}
      {:else}
        <a
          href={`/folders?path=${encodeURIComponent(node.path)}`}
          class="flex items-center gap-1.5 py-1 px-2 pl-7 rounded-lg hover:bg-white/5 transition-colors"
        >
          <Icon path={mdiFolderOutline} size={14} class="opacity-40 flex-none" />
          <span class="truncate text-[13px] leading-tight pr-2">{node.name}</span>
        </a>
      {/if}
    {/each}
  </div>
{/if}
