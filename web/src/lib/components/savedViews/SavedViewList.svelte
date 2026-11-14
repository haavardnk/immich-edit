<script lang="ts">
  import ListFilter from '#lib/components/library/ListFilter.svelte';
  import SavedViewItem from '#lib/components/savedViews/SavedViewItem.svelte';
  import type { SavedView } from '#lib/api/savedViews.ts';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';
  import { focusFirst, focusTargetsAround } from '#lib/utils/focusAround.ts';
  import { tick } from 'svelte';

  let { variant }: { variant: 'sidebar' | 'list' } = $props();

  const links = new Map<string, HTMLElement>();
  let list = $state<HTMLElement | null>(null);

  function register(id: string, node: HTMLElement): () => void {
    links.set(id, node);
    return () => {
      if (links.get(id) === node) links.delete(id);
    };
  }

  function refocusAfter(view: SavedView, visible: readonly SavedView[]): () => void {
    const index = visible.findIndex((v) => v.id === view.id);
    const neighbour = visible[index + 1] ?? visible[index - 1];
    if (neighbour) return () => links.get(neighbour.id)?.focus();
    const around = list ? focusTargetsAround(list) : [];
    return () => focusFirst(around);
  }

  async function remove(view: SavedView, visible: readonly SavedView[]): Promise<void> {
    const refocus = refocusAfter(view, visible);
    void savedViews.remove(view);
    await tick();
    requestAnimationFrame(refocus);
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
      <ul
        bind:this={list}
        aria-label="Saved views"
        class="flex flex-col {variant === 'sidebar' ? 'gap-1' : ''}"
      >
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
