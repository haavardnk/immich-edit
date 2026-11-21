<script lang="ts">
  import { onMount } from 'svelte';
  import LibraryLoad from '#lib/components/library/LibraryLoad.svelte';
  import TagList from '#lib/components/library/TagList.svelte';
  import PageFrame from '#lib/components/shell/PageFrame.svelte';
  import { userTags } from '#lib/browse/managedTags.ts';
  import { library } from '#lib/stores/library.svelte.ts';

  const TAG_NOUN = { one: 'tag', many: 'tags' };

  const count = $derived(
    library.isLoaded('tags') ? { value: userTags(library.tags).length, noun: TAG_NOUN } : undefined
  );

  onMount(() => {
    library.request('tags');
  });
</script>

<PageFrame title="Tags" {count}>
  <LibraryLoad view="tags" label="tags">
    <div class="max-w-2xl py-2 sm:px-3">
      <TagList />
    </div>
  </LibraryLoad>
</PageFrame>
