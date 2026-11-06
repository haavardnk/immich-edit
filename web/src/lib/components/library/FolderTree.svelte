<script lang="ts">
  import type { FolderNode } from '#lib/stores/library.svelte.ts';
  import { allFolders } from '#lib/browse/folders.ts';
  import FolderBranch from '#lib/components/library/FolderBranch.svelte';
  import FolderMatches from '#lib/components/library/FolderMatches.svelte';
  import ListFilter from '#lib/components/library/ListFilter.svelte';

  let { nodes }: { nodes: readonly FolderNode[] } = $props();

  let expanded = $state(new Set<string>());
  const folders = $derived(allFolders(nodes));

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
  <div class="px-5 py-3 text-xs text-muted" role="status">No folders</div>
{:else}
  <ListFilter
    items={folders}
    nameOf={(node) => node.name}
    noun={{ one: 'folder', many: 'folders' }}
    class="ps-4 pe-1 touch:px-4"
  >
    {#snippet children(visible, query)}
      {#if query}
        <FolderMatches folders={visible} {query} />
      {:else}
        <FolderBranch {nodes} {expanded} onToggle={toggle} />
      {/if}
    {/snippet}
  </ListFilter>
{/if}
