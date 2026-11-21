<script lang="ts">
  import { onMount } from 'svelte';
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import ListFilterField from '#lib/components/library/ListFilterField.svelte';
  import ListFilterResults from '#lib/components/library/ListFilterResults.svelte';
  import ListSortMenu from '#lib/components/library/ListSortMenu.svelte';
  import TagTreeView from '#lib/components/library/TagTreeView.svelte';
  import { TAG_TABLE_GRID } from '#lib/components/library/tagTable.ts';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { ListFilterState } from '#lib/browse/listFilter.svelte.ts';
  import { userTags } from '#lib/browse/managedTags.ts';
  import {
    TAG_SORTS,
    branchKeys,
    buildTagTree,
    sortTagTree,
    type TagSort
  } from '#lib/browse/tagTree.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { tagActions } from '#lib/stores/tagActions.svelte.ts';
  import { tagTreeOpen } from '#lib/stores/tagTreeOpen.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiPlus, mdiUnfoldLessHorizontal, mdiUnfoldMoreHorizontal } from '@mdi/js';

  const TAG_NOUN = { one: 'tag', many: 'tags' };

  let sort = $state<TagSort>('name');

  const loaded = $derived(library.isLoaded('tags'));
  const tags = $derived(userTags(library.tags));
  const nodes = $derived(sortTagTree(buildTagTree(tags), sort));
  const branches = $derived(branchKeys(nodes));
  const allOpen = $derived(tagTreeOpen.allOpen(branches));
  const foldLabel = $derived(allOpen ? 'Collapse all' : 'Expand all');

  const filter = new ListFilterState(
    () => tags,
    (tag) => tag.name,
    () => TAG_NOUN,
    { always: true }
  );

  function fold(): void {
    if (allOpen) tagTreeOpen.closeAll(branches);
    else tagTreeOpen.openAll(branches);
  }

  onMount(() => {
    library.request('tags');
  });
</script>

{#snippet actions()}
  {#if branches.length > 0 && !layout.compact}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:size-11"
      icon={allOpen ? mdiUnfoldLessHorizontal : mdiUnfoldMoreHorizontal}
      title={foldLabel}
      aria-label={foldLabel}
      onclick={fold}
    />
  {/if}
  <ListSortMenu noun="tags" options={TAG_SORTS} value={sort} onChange={(next) => (sort = next)} />
  <IconButton
    size="small"
    variant="ghost"
    color="secondary"
    class="touch:size-11"
    icon={mdiPlus}
    title="Create tag"
    aria-label="Create tag"
    onclick={() => void tagActions.create()}
  />
{/snippet}

{#snippet filterField()}
  <ListFilterField
    {filter}
    size="toolbar"
    class="ms-auto {layout.compact ? 'min-w-36 flex-1' : 'w-56 min-w-24 shrink'}"
  />
{/snippet}

<PageFrame
  title="Tags"
  count={loaded ? { value: tags.length, noun: TAG_NOUN } : undefined}
  {actions}
  toolbar={loaded && tags.length > 0 ? filterField : undefined}
>
  <LibraryLoad view="tags" label="tags">
    {#if tags.length === 0}
      <p class="px-6 py-16 text-center text-sm text-dark/65" role="status">No tags</p>
    {:else}
      <div class="px-2 py-2 sm:px-5 sm:py-4">
        <div
          class={layout.compact
            ? ''
            : 'overflow-hidden rounded-xl border border-hairline bg-light-50'}
        >
          {#if !layout.compact}
            <div
              class="section-label {TAG_TABLE_GRID} h-9 border-b border-hairline pe-2 ps-3 text-dark/65"
              aria-hidden="true"
            >
              <span>Name</span>
              <span class="text-end">Sub-tags</span>
              <span class="text-end">Photos</span>
              <span></span>
            </div>
          {/if}
          <ListFilterResults {filter} class="px-3">
            {#snippet children(visible, query)}
              <TagTreeView
                {nodes}
                matches={query ? visible : null}
                {query}
                density={layout.compact ? 'list' : 'table'}
              />
            {/snippet}
          </ListFilterResults>
        </div>
      </div>
    {/if}
  </LibraryLoad>
</PageFrame>
