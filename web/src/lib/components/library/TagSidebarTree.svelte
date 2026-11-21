<script lang="ts">
  import { userTags } from '#lib/browse/managedTags.ts';
  import { buildTagTree } from '#lib/browse/tagTree.ts';
  import ListFilter from '#lib/components/library/ListFilter.svelte';
  import TagTreeView from '#lib/components/library/TagTreeView.svelte';
  import { library } from '#lib/stores/library.svelte.ts';

  const tags = $derived(userTags(library.tags));
  const nodes = $derived(buildTagTree(tags));
</script>

{#if tags.length === 0}
  <div class="px-5 py-3 text-xs text-dark/65" role="status">No tags</div>
{:else}
  <ListFilter
    items={tags}
    nameOf={(tag) => tag.name}
    noun={{ one: 'tag', many: 'tags' }}
    class="ps-4 pe-1 touch:px-4"
  >
    {#snippet children(visible, query)}
      <TagTreeView {nodes} matches={query ? visible : null} {query} density="sidebar" />
    {/snippet}
  </ListFilter>
{/if}
