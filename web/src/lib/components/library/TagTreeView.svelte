<script lang="ts">
  import { page } from '$app/state';
  import type { TagSummary } from '#lib/api/tags.ts';
  import { matchedRows, tagNodeKey, treeRows, type TagNode } from '#lib/browse/tagTree.ts';
  import TagTreeItem from '#lib/components/library/TagTreeItem.svelte';
  import type { TagDensity } from '#lib/components/library/tagTable.ts';
  import { tagTreeOpen } from '#lib/stores/tagTreeOpen.svelte.ts';

  let {
    nodes,
    matches,
    query,
    density
  }: {
    nodes: readonly TagNode[];
    matches: readonly TagSummary[] | null;
    query: string;
    density: TagDensity;
  } = $props();

  const matched = $derived(matches ? new Set(matches.map((tag) => tag.id)) : null);
  const rows = $derived(
    matched ? matchedRows(nodes, matched, tagTreeOpen.isOpen) : treeRows(nodes, tagTreeOpen.isOpen)
  );
  const LIST: Readonly<Record<TagDensity, string>> = {
    table: 'divide-y divide-hairline',
    list: 'flex flex-col gap-px',
    sidebar: 'flex flex-col gap-0.5 p-1'
  };
  const activeId = $derived(
    page.url.pathname.startsWith('/tags/') ? (page.params.id ?? null) : null
  );
</script>

<ul class={LIST[density]}>
  {#each rows as row (row.node.path)}
    <TagTreeItem
      {row}
      {query}
      {density}
      active={row.node.tag !== null && row.node.tag.id === activeId}
      onToggle={() => tagTreeOpen.toggle(tagNodeKey(row.node))}
    />
  {/each}
</ul>
