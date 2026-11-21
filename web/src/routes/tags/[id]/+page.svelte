<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { tagTreeOpen } from '#lib/stores/tagTreeOpen.svelte.ts';
  import { readFilterParams } from '#lib/browse/filterParams.ts';
  import { userTags } from '#lib/browse/managedTags.ts';
  import {
    ancestorKeys,
    buildTagTree,
    findTagNode,
    splitTagValue,
    tagAncestors,
    tagPathLabel
  } from '#lib/browse/tagTree.ts';
  import { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { resumeId } from '#lib/browse/gridReturn.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import BrowsePage from '#lib/components/browse/BrowsePage.svelte';
  import TagSubtags from '#lib/components/library/TagSubtags.svelte';
  import { COLLECTIONS } from '#lib/components/library/collections.ts';

  const id = $derived(page.params.id as string);
  const feed = new BrowseFeed({ baseFilter: () => ({ scope: { tag: id } }) });
  const tag = $derived(library.tags.find((t) => t.id === id) ?? null);
  const tree = $derived(buildTagTree(userTags(library.tags)));
  const node = $derived(findTagNode(tree, id));
  const ancestors = $derived(tag ? tagAncestors(library.tags, tag) : []);
  const parents = $derived([
    COLLECTIONS.tags,
    ...ancestors.map((above) => ({ href: `/tags/${above.id}`, label: above.name }))
  ]);
  const context = $derived(tag ? tagPathLabel(splitTagValue(tag.value).parent) : '');

  $effect(() => {
    const _ = id;
    untrack(() => {
      selection.clear();
      library.request('tags');
      browseControls.enter('collection', readFilterParams(page.url));
      feed.reset();
      void feed.open(resumeId(page.url));
    });
    return () => feed.close();
  });

  let revealed: string | null = null;

  $effect(() => {
    const above = ancestorKeys(tree, id);
    if (!above || revealed === id) return;
    revealed = id;
    untrack(() => tagTreeOpen.openAll(above));
  });

  $effect(() => feed.watchFilterChange());
</script>

<BrowsePage
  {feed}
  {parents}
  context={context || undefined}
  title={tag ? splitTagValue(tag.value).leaf : 'Tag'}
  loadingLabel="Loading tag…"
  emptyTitle="No photos with this tag"
>
  {#snippet lead()}
    {#if node}
      <TagSubtags nodes={node.children} />
    {/if}
  {/snippet}
</BrowsePage>
