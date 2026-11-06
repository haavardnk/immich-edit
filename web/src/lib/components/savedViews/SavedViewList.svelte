<script lang="ts">
  import ListFilter from '#lib/components/library/ListFilter.svelte';
  import SavedViewItem from '#lib/components/savedViews/SavedViewItem.svelte';
  import type { SavedView } from '#lib/api/savedViews.ts';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';
  import { tick } from 'svelte';

  let { variant }: { variant: 'sidebar' | 'list' } = $props();

  const links = new Map<string, HTMLElement>();

  function register(id: string, node: HTMLElement): () => void {
    links.set(id, node);
    return () => {
      if (links.get(id) === node) links.delete(id);
    };
  }

  async function remove(view: SavedView, visible: readonly SavedView[]): Promise<void> {
    const index = visible.findIndex((v) => v.id === view.id);
    const neighbour = visible[index + 1] ?? visible[index - 1];
    void savedViews.remove(view);
    if (!neighbour) return;
    await tick();
    requestAnimationFrame(() => links.get(neighbour.id)?.focus());
  }
</script>

<ListFilter
  items={savedViews.views}
  nameOf={(view) => view.name}
  noun={{ one: 'view', many: 'views' }}
  class={variant === 'sidebar' ? 'ps-4 pe-1 wide:px-1' : 'px-4 sm:px-5'}
>
  {#snippet children(visible, query)}
    {#if visible.length > 0}
      <ul aria-label="Saved views" class="flex flex-col {variant === 'sidebar' ? 'gap-1' : ''}">
        {#each visible as view, index (view.id)}
          <li>
            <SavedViewItem
              {view}
              {query}
              {variant}
              last={index === visible.length - 1}
              onLink={(node) => register(view.id, node)}
              onDelete={() => void remove(view, visible)}
            />
          </li>
        {/each}
      </ul>
    {/if}
  {/snippet}
</ListFilter>
